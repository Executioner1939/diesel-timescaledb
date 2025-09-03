//! Basic usage example for diesel-timescaledb

use diesel::prelude::*;
use diesel_timescaledb::prelude::*;
use diesel_timescaledb::dsl::patterns::TimeSeriesAggregation;

// Example table schema (you would typically generate this with Diesel)
table! {
    metrics (id) {
        id -> Int4,
        timestamp -> Timestamptz,
        value -> Float8,
        device_id -> Text,
    }
}

// Implement Hypertable for the metrics table
hypertable!(metrics, timestamp);

fn main() {
    println!("diesel-timescaledb basic usage example");
    
    // Example of how you might use the types
    let now = TimestampTz::now();
    println!("Current timestamp: {:?}", now);
    
    // Example of using query patterns (with security validation)
    let agg = TimeSeriesAggregation::new(
        "metrics",
        "timestamp", 
        "value",
        "1 hour"
    ).expect("Invalid aggregation parameters");
    
    println!("Average query: {}", agg.avg_query());
    println!("Sum query: {}", agg.sum_query());
    println!("Count query: {}", agg.count_query());
    
    // Note: Actual database operations would require a connection
    // let mut conn = TimescaleDbConnection::establish("postgresql://...")
    //     .expect("Error connecting to database");
    // metrics::table::create_hypertable(&mut conn).expect("Error creating hypertable");
}