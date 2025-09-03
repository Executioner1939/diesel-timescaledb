//! # diesel-timescaledb
//!
//! A Diesel extension for TimescaleDB functionality.
//!
//! This crate provides Diesel-compatible types, functions, and utilities
//! for working with TimescaleDB's time-series database features.
//!
//! ## Quick Start
//!
//! ```rust
//! use diesel::prelude::*;
//! use diesel_timescaledb::prelude::*;
//!
//! table! {
//!     metrics (id) {
//!         id -> Int4,
//!         timestamp -> Timestamptz,
//!         value -> Float8,
//!     }
//! }
//!
//! // Make it a hypertable
//! hypertable!(metrics, timestamp);
//!
//! // Use TimescaleDB functions
//! // let results = metrics::table
//! //     .time_bucket(metrics::timestamp, "1 hour")
//! //     .load(&mut conn)?;
//! ```

pub mod connection;
pub mod dsl;
pub mod functions;
pub mod prelude;
pub mod schema;
pub mod types;

// Re-export commonly used items
pub use connection::TimescaleDbConnection;
pub use types::*;
