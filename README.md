# diesel-timescaledb

A pure Diesel extension for TimescaleDB functionality without telemetry.

## Overview

This crate provides Diesel-compatible types, functions, and utilities for working with TimescaleDB's time-series database features. It focuses purely on database functionality without any observability or telemetry features.

## Features

- **TimescaleDB Types**: Custom types for time-series data
- **SQL Functions**: Native TimescaleDB function bindings
- **Hypertable Support**: Utilities for creating and managing hypertables
- **Query DSL**: Time-series specific query builder extensions
- **Schema Utilities**: Macros and helpers for schema management

## Module Structure

- `connection`: TimescaleDB connection wrapper
- `types`: TimescaleDB-specific types and mappings
- `functions`: TimescaleDB SQL function definitions
- `dsl`: Query DSL extensions for time-series operations
- `schema`: Schema utilities and hypertable management

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
diesel-timescaledb = { path = "../Crates/diesel-timescaledb" }
```

## Example

```rust
use diesel_timescaledb::prelude::*;

// Create a hypertable
MyTable::create_hypertable(&mut conn)?;

// Use time bucket functions
let results = my_table
    .time_bucket(my_table::timestamp, "1 hour")
    .load(&mut conn)?;
```