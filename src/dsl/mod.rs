//! Domain Specific Language (DSL) extensions for TimescaleDB queries.

use crate::schema::{SqlIdentifier, TimeInterval, ValidationError};
use diesel::expression::AsExpression;
use diesel::prelude::*;
use diesel::sql_types::Timestamptz;

/// Extension trait for building time-series queries.
pub trait TimescaleQueryDsl: Sized {
    /// Add time bucketing to the query with a validated time interval.
    ///
    /// # Security
    /// Uses validated TimeInterval to prevent SQL injection.
    fn time_bucket<Expr>(
        self,
        time_column: Expr,
        interval: TimeInterval,
    ) -> TimeBucketQuery<Self, Expr>
    where
        Expr: Expression;

    /// Add time bucketing to the query with a string interval (legacy API).
    ///
    /// # Security
    /// Validates the interval string before using it.
    ///
    /// # Deprecated
    /// Use `time_bucket` with `TimeInterval` for better type safety.
    fn time_bucket_str<Expr>(
        self,
        time_column: Expr,
        interval: &str,
    ) -> Result<TimeBucketQuery<Self, Expr>, ValidationError>
    where
        Expr: Expression;

    /// Add a time range filter to the query.
    fn time_range<Expr, V>(
        self,
        time_column: Expr,
        start: V,
        end: V,
    ) -> TimeRangeQuery<Self, Expr, V>
    where
        Expr: Expression,
        V: AsExpression<Timestamptz>;
}

impl<T> TimescaleQueryDsl for T {
    fn time_bucket<Expr>(
        self,
        time_column: Expr,
        interval: TimeInterval,
    ) -> TimeBucketQuery<Self, Expr>
    where
        Expr: Expression,
    {
        TimeBucketQuery {
            query: self,
            time_column,
            interval,
        }
    }

    fn time_bucket_str<Expr>(
        self,
        time_column: Expr,
        interval: &str,
    ) -> Result<TimeBucketQuery<Self, Expr>, ValidationError>
    where
        Expr: Expression,
    {
        let validated_interval = TimeInterval::from_string(interval)?;
        Ok(TimeBucketQuery {
            query: self,
            time_column,
            interval: validated_interval,
        })
    }

    fn time_range<Expr, V>(
        self,
        time_column: Expr,
        start: V,
        end: V,
    ) -> TimeRangeQuery<Self, Expr, V>
    where
        Expr: Expression,
        V: AsExpression<Timestamptz>,
    {
        TimeRangeQuery {
            query: self,
            time_column,
            start,
            end,
        }
    }
}

/// A query with time bucketing applied.
#[derive(Debug, Clone)]
pub struct TimeBucketQuery<Query, TimeColumn> {
    query: Query,
    time_column: TimeColumn,
    interval: TimeInterval,
}

impl<Query, TimeColumn> TimeBucketQuery<Query, TimeColumn> {
    /// Get the underlying query.
    pub fn into_inner(self) -> Query {
        self.query
    }

    /// Get the time column expression.
    pub fn time_column(&self) -> &TimeColumn {
        &self.time_column
    }

    /// Get the interval.
    pub fn interval(&self) -> &TimeInterval {
        &self.interval
    }

    /// Get the interval as a PostgreSQL interval string.
    pub fn interval_sql(&self) -> String {
        self.interval.to_postgres_interval()
    }
}

/// A query with time range filtering applied.
#[derive(Debug, Clone)]
pub struct TimeRangeQuery<Query, TimeColumn, Value> {
    query: Query,
    time_column: TimeColumn,
    start: Value,
    end: Value,
}

impl<Query, TimeColumn, Value> TimeRangeQuery<Query, TimeColumn, Value> {
    /// Get the underlying query.
    pub fn into_inner(self) -> Query {
        self.query
    }

    /// Get the time column expression.
    pub fn time_column(&self) -> &TimeColumn {
        &self.time_column
    }

    /// Get the start time value.
    pub fn start(&self) -> &Value {
        &self.start
    }

    /// Get the end time value.
    pub fn end(&self) -> &Value {
        &self.end
    }
}

/// Trait for queries that can be executed with time-series optimizations.
pub trait TimescaleExecuteDsl<Conn> {
    /// Execute the query with TimescaleDB optimizations enabled.
    fn execute_timescale(self, conn: &mut Conn) -> QueryResult<usize>;
}

/// Common time-series query patterns.
pub mod patterns {
    use super::*;

    /// Helper for creating common time-series aggregation queries.
    /// This version ensures type safety and prevents SQL injection.
    pub struct TimeSeriesAggregation {
        pub table_name: SqlIdentifier,
        pub time_column: SqlIdentifier,
        pub value_column: SqlIdentifier,
        pub bucket_interval: TimeInterval,
    }

    impl TimeSeriesAggregation {
        /// Create a new time-series aggregation with validated inputs.
        ///
        /// # Security
        /// All parameters are validated to prevent SQL injection attacks.
        pub fn new(
            table_name: &str,
            time_column: &str,
            value_column: &str,
            bucket_interval: &str,
        ) -> Result<Self, ValidationError> {
            Ok(Self {
                table_name: SqlIdentifier::new(table_name)?,
                time_column: SqlIdentifier::new(time_column)?,
                value_column: SqlIdentifier::new(value_column)?,
                bucket_interval: TimeInterval::from_string(bucket_interval)?,
            })
        }

        /// Create a new time-series aggregation with type-safe inputs.
        pub fn new_typed(
            table_name: SqlIdentifier,
            time_column: SqlIdentifier,
            value_column: SqlIdentifier,
            bucket_interval: TimeInterval,
        ) -> Self {
            Self {
                table_name,
                time_column,
                value_column,
                bucket_interval,
            }
        }

        /// Build a query string for average aggregation.
        ///
        /// # Security
        /// All identifiers are properly escaped to prevent SQL injection.
        pub fn avg_query(&self) -> String {
            format!(
                "SELECT time_bucket(INTERVAL '{}', {}) as bucket, avg({}) as average 
                 FROM {} 
                 GROUP BY bucket 
                 ORDER BY bucket",
                self.bucket_interval.to_postgres_interval(),
                self.time_column.escaped(),
                self.value_column.escaped(),
                self.table_name.escaped()
            )
        }

        /// Build a query string for sum aggregation.
        ///
        /// # Security
        /// All identifiers are properly escaped to prevent SQL injection.
        pub fn sum_query(&self) -> String {
            format!(
                "SELECT time_bucket(INTERVAL '{}', {}) as bucket, sum({}) as total 
                 FROM {} 
                 GROUP BY bucket 
                 ORDER BY bucket",
                self.bucket_interval.to_postgres_interval(),
                self.time_column.escaped(),
                self.value_column.escaped(),
                self.table_name.escaped()
            )
        }

        /// Build a query string for count aggregation.
        ///
        /// # Security
        /// All identifiers are properly escaped to prevent SQL injection.
        pub fn count_query(&self) -> String {
            format!(
                "SELECT time_bucket(INTERVAL '{}', {}) as bucket, count(*) as count 
                 FROM {} 
                 GROUP BY bucket 
                 ORDER BY bucket",
                self.bucket_interval.to_postgres_interval(),
                self.time_column.escaped(),
                self.table_name.escaped()
            )
        }
    }
}
