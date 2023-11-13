use parser::types::{DocumentOperations, ExecutableDocument, OperationType, Selection::Field};
use value::Variables;

#[derive(Debug)]
pub struct ProcessedQuery<'a> {
    document: &'a ExecutableDocument,
    operation_type: OperationType,
    operation_name: String,
    variables: &'a Variables,
}

impl<'a> ProcessedQuery<'a> {
    pub fn try_new(
        document: &'a ExecutableDocument,
        variables: &'a Variables,
    ) -> anyhow::Result<Self> {
        let operation = match &document.operations {
            DocumentOperations::Single(operation) => Some(operation),
            DocumentOperations::Multiple(map) if map.len() == 1 => {
                Some(map.iter().next().unwrap().1)
            }
            DocumentOperations::Multiple(_) => None,
        }
        .ok_or(anyhow::anyhow!("No operation found"))?;

        let field = operation
            .node
            .selection_set
            .node
            .items
            .iter()
            .find(|selection| matches!(selection.node, Field(_)))
            .and_then(|selection| match &selection.node {
                Field(field) => Some(field),
                _ => None,
            })
            .map(|field| field.node.name.to_string())
            .ok_or(anyhow::anyhow!("No field found."))?;

        Ok(Self {
            document,
            operation_type: operation.node.ty,
            operation_name: field,
            variables,
        })
    }

    pub fn is_introspection(&self) -> bool {
        self.operation_name == "__schema" || self.operation_name == "__type"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single() {
        let query = r#"
            query {
                hello
            }
        "#;

        let document = parser::parse_query(query).expect("Failed to parse query");
        let variables = Variables::default();

        let q =
            ProcessedQuery::try_new(&document, &variables).expect("Failed to create query details");
        assert_eq!(q.operation_type, OperationType::Query);
        assert_eq!(q.operation_name, "hello");
        assert_eq!(q.variables.len(), 0);
    }

    #[test]
    fn parse_multiple() {
        let query = r#"
            query GetBanner($id: ID) {
                banner(id: $id) {
                    id
                    link {
                        locale
                    }
                    createdAt
                }
            }
        "#;

        let document = parser::parse_query(query).expect("Failed to parse query");
        let variables = Variables::from_json(
            serde_json::from_str(r#"{"id": "1"}"#).expect("Failed to parse variables"),
        );
        let q = ProcessedQuery::try_new(&document, &variables)
            .expect("Failed to retrieve query details");

        assert_eq!(q.operation_type, OperationType::Query);
        assert_eq!(q.operation_name, "banner");
        assert_eq!(q.variables.len(), 1);
    }

    #[test]
    fn parse_introspection_schema() {
        let query = r#"
            {
              __schema {
                types {
                  name
                }
              }
            }
        "#;

        let document = parser::parse_query(query).expect("Failed to parse query");
        let variables = Variables::default();
        let q = ProcessedQuery::try_new(&document, &variables)
            .expect("Failed to retrieve query details.");

        assert_eq!(q.operation_type, OperationType::Query);
        assert_eq!(q.operation_name, "__schema");
        assert_eq!(q.variables.len(), 0);
        assert!(q.is_introspection());
    }

    #[test]
    fn parse_introspection_type() {
        let query = r#"
            {
              __type(name: "Droid") {
                name
              }
            }
        "#;

        let document = parser::parse_query(query).expect("Failed to parse query.");
        let variables = Variables::default();
        let q = ProcessedQuery::try_new(&document, &variables)
            .expect("Failed to retrieve query details.");

        assert_eq!(q.operation_type, OperationType::Query);
        assert_eq!(q.operation_name, "__type");
        assert_eq!(q.variables.len(), 0);
        assert!(q.is_introspection());
    }
}
