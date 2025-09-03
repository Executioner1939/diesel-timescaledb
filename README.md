# diesel-timescaledb

[![Crates.io](https://img.shields.io/crates/v/diesel-timescaledb.svg)](https://crates.io/crates/diesel-timescaledb)
[![Documentation](https://docs.rs/diesel-timescaledb/badge.svg)](https://docs.rs/diesel-timescaledb)
[![License](https://img.shields.io/crates/l/diesel-timescaledb)](https://github.com/diesel-timescaledb/diesel-timescaledb)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/diesel-timescaledb/diesel-timescaledb)
[![Minimum rustc version](https://img.shields.io/badge/rustc-1.65+-lightgray.svg)](https://github.com/diesel-timescaledb/diesel-timescaledb)
[![Diesel Version](https://img.shields.io/badge/diesel-2.1-blue)](https://diesel.rs)

A pure Diesel extension crate that provides comprehensive TimescaleDB functionality for Rust applications. This crate seamlessly integrates TimescaleDB's powerful time-series capabilities with Diesel's type-safe query builder, offering a robust solution for time-series data management.

## Features

- 🚀 **Native TimescaleDB Integration** - Full support for TimescaleDB-specific functions and features
- 🔒 **Type-Safe Operations** - Leverage Diesel's compile-time guarantees for database operations
- 📊 **Time-Series Functions** - Complete bindings for TimescaleDB's time-series SQL functions
- 🏗️ **Hypertable Management** - Easy creation and management of hypertables
- 🎯 **Query DSL Extensions** - Time-series specific query builder extensions
- ⚡ **Zero Overhead** - Pure database functionality without telemetry overhead
- 🛠️ **Schema Utilities** - Comprehensive macros and helpers for schema management

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
diesel = { version = "2.1", features = ["postgres", "chrono"] }
diesel-timescaledb = "0.1.0"
```

### Feature Flags

This crate currently has no feature flags. All functionality is included by default and requires:
- `diesel` with `postgres` and `chrono` features
- PostgreSQL with TimescaleDB extension installed

## Quick Start

```rust
use diesel::prelude::*;
use diesel_timescaledb::prelude::*;

// Define your table schema
table! {
    metrics (id) {
        id -> Int4,
        timestamp -> Timestamptz,
        value -> Float8,
        device_id -> Text,
    }
}

// Make it a hypertable
hypertable!(metrics, timestamp);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Establish connection
    let mut conn = TimescaleDbConnection::establish("postgresql://user:pass@localhost/tsdb")?;
    
    // Create hypertable
    metrics::table::create_hypertable(&mut conn)?;
    
    // Use time bucket for aggregation
    let hourly_averages = metrics::table
        .select((
            time_bucket(Interval::from_hours(1), metrics::timestamp),
            diesel::dsl::avg(metrics::value),
        ))
        .group_by(time_bucket(Interval::from_hours(1), metrics::timestamp))
        .load::<(TimestampTz, Option<f64>)>(&mut conn)?;
    
    Ok(())
}
```

## API Overview

### Core Modules

#### `connection`
Provides the `TimescaleDbConnection` wrapper that extends Diesel's `PgConnection` with TimescaleDB-specific capabilities.

```rust
use diesel_timescaledb::connection::TimescaleDbConnection;

let mut conn = TimescaleDbConnection::establish("postgresql://...")?;
```

#### `types`
TimescaleDB-specific type mappings for seamless integration with Rust's type system.

```rust
use diesel_timescaledb::types::{TimestampTz, TimeDimension};

// Work with timezone-aware timestamps
let now = TimestampTz::now();

// Define time dimensions for continuous aggregates
let dimension = TimeDimension::new("timestamp", "1 hour");
```

#### `functions`
Complete set of TimescaleDB SQL functions available through Diesel's expression system.

```rust
use diesel_timescaledb::functions::*;

// Time bucketing
time_bucket(interval, timestamp_column)
time_bucket_with_origin(interval, timestamp_column, origin)

// Gapfilling
time_bucket_gapfill(interval, timestamp_column)
locf(value_column)  // Last observation carried forward
interpolate(value_column)

// Statistics
first(value, timestamp)  // First value in time range
last(value, timestamp)   // Last value in time range
histogram(value, min, max, buckets)

// Approximation functions
approx_percentile(value, percentile)
percentile_agg(value)
```

#### `dsl`
Query DSL extensions that make time-series queries more ergonomic.

```rust
use diesel_timescaledb::dsl::*;

// Use the TimeSeriesAggregation pattern
let aggregation = TimeSeriesAggregation::new(
    "metrics",
    "timestamp",
    "value",
    "1 hour"
)?;

// Chain time-series specific operations
let result = metrics::table
    .time_bucket(metrics::timestamp, "1 hour")
    .filter(metrics::timestamp.gt(now - 1.week()))
    .load(&mut conn)?;
```

#### `schema`
Utilities for managing TimescaleDB schema objects like hypertables and continuous aggregates.

```rust
use diesel_timescaledb::schema::*;

// Configure continuous aggregates
let config = ContinuousAggregateConfig {
    name: "hourly_metrics",
    view_query: "SELECT time_bucket('1 hour', timestamp), avg(value) FROM metrics",
    materialized_only: false,
    refresh_interval: Some("1 hour"),
};

// Create the continuous aggregate
create_continuous_aggregate(&mut conn, config)?;
```

### Macros

#### `hypertable!`
Macro for declaring a table as a hypertable and generating helper methods.

```rust
// Declare a hypertable
hypertable!(metrics, timestamp);

// This generates:
// - metrics::table::create_hypertable(&mut conn)
// - metrics::table::drop_hypertable(&mut conn)
// - metrics::table::add_compression_policy(&mut conn, "7 days")
// - metrics::table::add_retention_policy(&mut conn, "1 year")
```

## Advanced Usage

### Working with Continuous Aggregates

```rust
use diesel_timescaledb::prelude::*;

// Define a continuous aggregate for hourly statistics
let hourly_stats = ContinuousAggregateConfig {
    name: "metrics_hourly",
    view_query: r#"
        SELECT 
            time_bucket('1 hour', timestamp) AS hour,
            device_id,
            avg(value) as avg_value,
            max(value) as max_value,
            min(value) as min_value,
            count(*) as sample_count
        FROM metrics
        GROUP BY hour, device_id
    "#,
    materialized_only: false,
    refresh_interval: Some("30 minutes"),
};

// Create the aggregate
create_continuous_aggregate(&mut conn, hourly_stats)?;

// Query the aggregate
table! {
    metrics_hourly (hour, device_id) {
        hour -> Timestamptz,
        device_id -> Text,
        avg_value -> Float8,
        max_value -> Float8,
        min_value -> Float8,
        sample_count -> Int8,
    }
}

let recent_stats = metrics_hourly::table
    .filter(metrics_hourly::hour.gt(now - 1.day()))
    .load(&mut conn)?;
```

### Compression Policies

```rust
// Add compression policy to compress chunks older than 7 days
metrics::table::add_compression_policy(&mut conn, "7 days")?;

// Manually compress specific chunks
compress_chunk(&mut conn, "metrics", older_than = "30 days")?;
```

### Data Retention

```rust
// Automatically drop data older than 1 year
metrics::table::add_retention_policy(&mut conn, "1 year")?;

// Configure cascading policies
let retention_config = RetentionPolicy {
    table: "metrics",
    drop_after: "1 year",
    cascade_to_continuous_aggregates: true,
};

apply_retention_policy(&mut conn, retention_config)?;
```

### Gapfilling Queries

```rust
use diesel_timescaledb::functions::*;

// Fill gaps in time-series data
let filled_data = metrics::table
    .select((
        time_bucket_gapfill(
            Interval::from_hours(1),
            metrics::timestamp,
            now - 1.day(),
            now,
        ),
        locf(diesel::dsl::avg(metrics::value)),  // Last observation carried forward
    ))
    .group_by(time_bucket_gapfill(
        Interval::from_hours(1),
        metrics::timestamp,
        now - 1.day(),
        now,
    ))
    .load::<(TimestampTz, Option<f64>)>(&mut conn)?;
```

### Working with Time Zones

```rust
use diesel_timescaledb::types::TimestampTz;
use chrono::{DateTime, Utc, TimeZone};
use chrono_tz::US::Pacific;

// Convert between time zones
let utc_time = TimestampTz::now();
let pacific_time = Pacific.from_utc_datetime(&utc_time.naive_utc());

// Query with timezone-aware timestamps
let results = metrics::table
    .filter(metrics::timestamp.between(
        TimestampTz::from_utc(start_time),
        TimestampTz::from_utc(end_time),
    ))
    .load(&mut conn)?;
```

## Performance Considerations

### Chunk Size Optimization

```rust
// Configure chunk time interval for optimal performance
alter_chunk_time_interval(&mut conn, "metrics", "6 hours")?;
```

### Index Management

```rust
// Create optimized indexes for time-series queries
diesel::sql_query(
    "CREATE INDEX ON metrics (device_id, timestamp DESC) WHERE timestamp > NOW() - INTERVAL '7 days'"
).execute(&mut conn)?;
```

### Query Optimization Tips

1. **Use time_bucket for aggregations** - More efficient than GROUP BY with date_trunc
2. **Leverage continuous aggregates** - Pre-compute common aggregations
3. **Apply WHERE clauses on time** - Helps with chunk exclusion
4. **Use compression** - Reduces storage and can improve query performance
5. **Partition by time first** - Take advantage of hypertable partitioning

## Requirements

- Rust 1.65 or later
- PostgreSQL 12+ with TimescaleDB 2.0+ extension
- Diesel 2.1+ with PostgreSQL backend

### Database Setup

```sql
-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Verify installation
SELECT extversion FROM pg_extension WHERE extname = 'timescaledb';
```

## Error Handling

All operations return `Result` types with detailed error information:

```rust
use diesel_timescaledb::error::TimescaleDbError;

match metrics::table::create_hypertable(&mut conn) {
    Ok(_) => println!("Hypertable created successfully"),
    Err(TimescaleDbError::AlreadyExists) => println!("Hypertable already exists"),
    Err(e) => eprintln!("Error creating hypertable: {}", e),
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Executioner1939/diesel-timescaledb.git

# Run tests
cargo test

# Run tests with all features
cargo test --all-features

# Build documentation
cargo doc --open

# Run linting
cargo clippy --all-targets --all-features
```

### Testing

Tests require a PostgreSQL database with TimescaleDB installed. Set the `DATABASE_URL` environment variable:

```bash
export DATABASE_URL=postgresql://user:password@localhost/test_db
cargo test
```

## Examples

Check out the [examples](examples/) directory for more comprehensive examples:

- `basic_usage.rs` - Getting started with hypertables and time bucketing
- `continuous_aggregates.rs` - Working with continuous aggregates
- `compression.rs` - Implementing compression strategies
- `gapfilling.rs` - Handling missing data points

Run examples with:

```bash
cargo run --example basic_usage
```

## Related Projects

- [Diesel](https://diesel.rs) - A safe, extensible ORM and Query Builder for Rust
- [TimescaleDB](https://www.timescale.com) - Time-series database built on PostgreSQL
- [otel-instrumentation-diesel](../otel-instrumentation-diesel) - OpenTelemetry instrumentation for Diesel

## License

This project is dual-licensed under either:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

## Support

For issues and questions:
- Documentation: [docs.rs/diesel-timescaledb](https://docs.rs/diesel-timescaledb)

---
