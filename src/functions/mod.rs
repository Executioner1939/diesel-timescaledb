//! # Documentation for SQL Functions and Utilities in the Module
//!

use diesel::expression::functions::define_sql_function;
use diesel::sql_types::*;

// Time bucket functions for aggregating time-series data
define_sql_function! {
    /// Groups timestamps into buckets of a specified interval.
    /// 
    /// This is one of TimescaleDB's most important functions for time-series aggregation.
    /// 
    /// # Example SQL
    /// ```sql
    /// SELECT time_bucket('1 hour', timestamp_col), avg(value)
    /// FROM metrics
    /// GROUP BY time_bucket('1 hour', timestamp_col);
    /// ```
    fn time_bucket(interval: Interval, timestamp: Timestamptz) -> Timestamptz;
}

define_sql_function! {
    /// Groups timestamps into buckets with a specified origin.
    fn time_bucket_with_origin(interval: Interval, timestamp: Timestamptz, origin: Timestamptz) -> Timestamptz;
}

define_sql_function! {
    /// Groups integer values into buckets.
    fn time_bucket_int(bucket_width: Integer, timestamp: Bigint) -> Bigint;
}

// Hypertable management functions
define_sql_function! {
    /// Creates a hypertable from a regular PostgreSQL table.
    fn create_hypertable(relation: Text, time_column_name: Text) -> Bool;
}

define_sql_function! {
    /// Creates a hypertable with additional options.
    fn create_hypertable_with_options(
        relation: Text, 
        time_column_name: Text, 
        partitioning_column: Nullable<Text>,
        number_partitions: Nullable<Integer>,
        chunk_time_interval: Nullable<Interval>
    ) -> Bool;
}

// Continuous aggregates functions
define_sql_function! {
    /// Refreshes a continuous aggregate.
    fn refresh_continuous_aggregate(view_name: Text, window_start: Nullable<Timestamptz>, window_end: Nullable<Timestamptz>);
}

// Compression functions
define_sql_function! {
    /// Enables compression on a hypertable.
    fn add_compression_policy(hypertable: Text, compress_after: Interval) -> Integer;
}

define_sql_function! {
    /// Manually compresses chunks older than a specified time.
    fn compress_chunk(chunk_schema: Text, chunk_name: Text) -> Bool;
}

// Data retention functions
define_sql_function! {
    /// Adds a retention policy to automatically drop old data.
    fn add_retention_policy(hypertable: Text, drop_after: Interval) -> Integer;
}

define_sql_function! {
    /// Manually drops chunks older than a specified time.
    fn drop_chunks(relation: Text, older_than: Timestamptz);
}

// Statistical and analytical functions
define_sql_function! {
    /// Calculates the first value in a time-ordered set for numeric values.
    fn first_numeric(value: Double, time: Timestamptz) -> Nullable<Double>;
}

define_sql_function! {
    /// Calculates the last value in a time-ordered set for numeric values.
    fn last_numeric(value: Double, time: Timestamptz) -> Nullable<Double>;
}

define_sql_function! {
    /// Calculates the first value in a time-ordered set for integer values.
    fn first_integer(value: Integer, time: Timestamptz) -> Nullable<Integer>;
}

define_sql_function! {
    /// Calculates the last value in a time-ordered set for integer values.
    fn last_integer(value: Integer, time: Timestamptz) -> Nullable<Integer>;
}

define_sql_function! {
    /// Calculates a histogram of values.
    fn histogram(value: Double, min_val: Double, max_val: Double, num_buckets: Integer) -> Array<Integer>;
}

/// The `utilities` module contains utility functions and helpers for database operations
/// using Diesel ORM. It provides reusable abstractions and SQL expressions to make
/// interacting with the database more convenient.
pub mod utilities {
    use super::*;
    use diesel::prelude::*;
    use diesel::expression::SqlLiteral;
    use crate::schema::{TimeInterval, ValidationError};
    
    /// Creates a `time_bucket` SQL expression for aggregating timestamps into
    /// buckets of the specified interval.
    ///
    /// This function generates an expression equivalent to using the `time_bucket`
    /// function in PostgreSQL, which returns the timestamp rounded down to the
    /// nearest bucket boundary defined by the given interval. The function is
    /// primarily used in time-series data queries for bucketing timestamps.
    ///
    /// # Parameters
    ///
    /// - `interval`: A `TimeInterval` that specifies the duration of each bucket.
    ///               This will be converted into a PostgreSQL interval for use
    ///               in the query.
    /// - `timestamp_expr`: A timestamp expression of type `T` that will be
    ///                     bucketed. This expression must have the `Timestamptz`
    ///                     SQL type.
    ///
    /// # Returns
    ///
    /// A `time_bucket` expression, which is constructed using the `diesel`
    /// library's DSL. The returned expression has a SQL type of `Timestamptz`
    /// and can be used within a Diesel query.
    ///
    /// # Constraints
    ///
    /// - `T`: Must implement the `Expression` trait and have a `SqlType` of `Timestamptz`.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// # use diesel::dsl::*;
    /// # use your_crate::*;
    /// let interval = TimeInterval::from_minutes(15);
    /// let timestamp_expr = your_column; // Example for a Diesel column expression
    ///
    /// let bucketed_expr = time_bucket_expr(interval, timestamp_expr);
    /// // Now you can use `bucketed_expr` in your query
    /// ```
    ///
    /// # Notes
    ///
    /// - The `interval` is converted to a PostgreSQL-compatible interval string
    ///   using `to_postgres_interval()`.
    /// - Ensure the target database is PostgreSQL, as `time_bucket` is specific
    ///   to PostgreSQL and extensions like TimescaleDB.
    ///
    /// # Dependencies
    ///
    /// This function relies on the `diesel` crate for SQL generation and the
    /// appropriate traits and types being in scope.
    pub fn time_bucket_expr<T>(interval: TimeInterval, timestamp_expr: T) -> time_bucket<SqlLiteral<Interval>, T> 
    where
        T: Expression<SqlType = Timestamptz>,
    {
        time_bucket(
            diesel::dsl::sql::<Interval>(&format!("INTERVAL '{}'", interval.to_postgres_interval())), 
            timestamp_expr
        )
    }
}