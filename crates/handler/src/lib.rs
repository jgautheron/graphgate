#![forbid(unsafe_code)]

pub use service_route::{ServiceRoute, ServiceRouteTable};
pub use shared_route_table::{SharedConfig, SharedRouteTable};

pub mod auth;
mod constants;
mod executor;
mod fetcher;
mod introspection;
mod metrics;
mod service_route;
mod shared_route_table;
mod websocket;

pub mod handler;
mod query;
