//! Schema management for TimescaleDB hypertables and related structures.

use diesel::prelude::*;
use diesel::sql_types::{Nullable, Text, Timestamptz};
use std::fmt;

/// A validated SQL identifier that prevents SQL injection attacks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlIdentifier(String);

impl SqlIdentifier {
    /// Create a new validated SQL identifier.
    pub fn new(identifier: &str) -> Result<Self, ValidationError> {
        validate_sql_identifier(identifier)?;
        Ok(SqlIdentifier(identifier.to_string()))
    }

    /// Get the escaped identifier suitable for use in SQL queries.
    pub fn escaped(&self) -> String {
        format!("\"{}\"", self.0.replace('"', "\"\""))
    }

    /// Get the raw identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SqlIdentifier {
    /// Format the identifier for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.escaped())
    }
}

/// Represents a time interval for TimescaleDB operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeInterval {
    value: u64,
    unit: TimeUnit,
}

/// Units of time for intervals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimeUnit {
    Microseconds,
    Milliseconds,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

impl TimeInterval {
    /// Create a new time interval.
    pub fn new(value: u64, unit: TimeUnit) -> Self {
        Self { value, unit }
    }

    /// Convert to a PostgreSQL interval string.
    pub fn to_postgres_interval(&self) -> String {
        let unit_str = match self.unit {
            TimeUnit::Microseconds => "microseconds",
            TimeUnit::Milliseconds => "milliseconds",
            TimeUnit::Seconds => "seconds",
            TimeUnit::Minutes => "minutes",
            TimeUnit::Hours => "hours",
            TimeUnit::Days => "days",
            TimeUnit::Weeks => "weeks",
            TimeUnit::Months => "months",
            TimeUnit::Years => "years",
        };
        format!("{} {}", self.value, unit_str)
    }

    /// Parse a time interval from a string.
    pub fn from_string(interval: &str) -> Result<Self, ValidationError> {
        validate_interval_string(interval)?;

        let parts: Vec<&str> = interval.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(ValidationError::InvalidInterval(
                "Interval must have format 'number unit'".to_string(),
            ));
        }

        let value: u64 = parts[0]
            .parse()
            .map_err(|_| ValidationError::InvalidInterval("Invalid numeric value".to_string()))?;

        let unit = match parts[1].to_lowercase().as_str() {
            "microsecond" | "microseconds" | "us" => TimeUnit::Microseconds,
            "millisecond" | "milliseconds" | "ms" => TimeUnit::Milliseconds,
            "second" | "seconds" | "s" => TimeUnit::Seconds,
            "minute" | "minutes" | "m" => TimeUnit::Minutes,
            "hour" | "hours" | "h" => TimeUnit::Hours,
            "day" | "days" | "d" => TimeUnit::Days,
            "week" | "weeks" | "w" => TimeUnit::Weeks,
            "month" | "months" => TimeUnit::Months,
            "year" | "years" | "y" => TimeUnit::Years,
            _ => {
                return Err(ValidationError::InvalidInterval(format!(
                    "Unknown time unit: {}",
                    parts[1]
                )))
            }
        };

        Ok(TimeInterval::new(value, unit))
    }
}

/// Validation error types for SQL identifiers and intervals.
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidIdentifier(String),
    InvalidInterval(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidIdentifier(msg) => write!(f, "Invalid SQL identifier: {}", msg),
            ValidationError::InvalidInterval(msg) => write!(f, "Invalid time interval: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate a SQL identifier to prevent SQL injection.
fn validate_sql_identifier(identifier: &str) -> Result<(), ValidationError> {
    if identifier.is_empty() {
        return Err(ValidationError::InvalidIdentifier(
            "Identifier cannot be empty".to_string(),
        ));
    }

    if identifier.len() > 63 {
        return Err(ValidationError::InvalidIdentifier(
            "Identifier too long (max 63 characters)".to_string(),
        ));
    }

    // PostgreSQL identifier rules: start with letter or underscore, followed by letters, digits, underscores, or dollar signs
    let first_char = identifier.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return Err(ValidationError::InvalidIdentifier(
            "Identifier must start with letter or underscore".to_string(),
        ));
    }

    for c in identifier.chars() {
        if !c.is_ascii_alphanumeric() && c != '_' && c != '$' {
            return Err(ValidationError::InvalidIdentifier(format!(
                "Invalid character '{}' in identifier",
                c
            )));
        }
    }

    // Check for SQL reserved words (basic list - could be extended)
    let reserved_words = [
        "select",
        "insert",
        "update",
        "delete",
        "drop",
        "create",
        "alter",
        "table",
        "index",
        "view",
        "procedure",
        "function",
        "trigger",
        "from",
        "where",
        "join",
        "union",
        "order",
        "group",
        "having",
        "and",
        "or",
        "not",
        "null",
        "true",
        "false",
    ];

    if reserved_words.contains(&identifier.to_lowercase().as_str()) {
        return Err(ValidationError::InvalidIdentifier(format!(
            "'{}' is a reserved SQL keyword",
            identifier
        )));
    }

    Ok(())
}

/// Validate an interval string to prevent SQL injection.
fn validate_interval_string(interval: &str) -> Result<(), ValidationError> {
    if interval.is_empty() {
        return Err(ValidationError::InvalidInterval(
            "Interval cannot be empty".to_string(),
        ));
    }

    if interval.len() > 50 {
        return Err(ValidationError::InvalidInterval(
            "Interval string too long".to_string(),
        ));
    }

    // Only allow alphanumeric characters, spaces, and basic punctuation
    for c in interval.chars() {
        if !c.is_ascii_alphanumeric() && !" .-_".contains(c) {
            return Err(ValidationError::InvalidInterval(format!(
                "Invalid character '{}' in interval",
                c
            )));
        }
    }

    Ok(())
}

/// Trait for tables that can be converted to TimescaleDB hypertables.
pub trait Hypertable {
    /// Name of the table to convert to a hypertable.
    const TABLE_NAME: &'static str;

    /// Name of the time column to use for partitioning.
    const TIME_COLUMN: &'static str;

    /// Create a hypertable from this table.
    fn create_hypertable(conn: &mut PgConnection) -> QueryResult<()> {
        // These are compile-time constants, so they're safe to use directly
        let query = "SELECT create_hypertable($1, $2);";

        diesel::sql_query(query)
            .bind::<Text, _>(Self::TABLE_NAME)
            .bind::<Text, _>(Self::TIME_COLUMN)
            .execute(conn)?;
        Ok(())
    }

    /// Create a hypertable with a specific chunk time interval.
    fn create_hypertable_with_interval(
        conn: &mut PgConnection,
        chunk_time_interval: TimeInterval,
    ) -> QueryResult<()> {
        let query = format!(
            "SELECT create_hypertable($1, $2, chunk_time_interval => INTERVAL '{}');",
            chunk_time_interval.to_postgres_interval()
        );

        diesel::sql_query(query)
            .bind::<Text, _>(Self::TABLE_NAME)
            .bind::<Text, _>(Self::TIME_COLUMN)
            .execute(conn)?;
        Ok(())
    }

    /// Create a hypertable with a specific chunk time interval from a string.
    fn create_hypertable_with_interval_str(
        conn: &mut PgConnection,
        chunk_time_interval: &str,
    ) -> QueryResult<()> {
        let interval = TimeInterval::from_string(chunk_time_interval).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        Self::create_hypertable_with_interval(conn, interval)
    }

    /// Add a compression policy to the hypertable.
    fn add_compression_policy(
        conn: &mut PgConnection,
        compress_after: TimeInterval,
    ) -> QueryResult<()> {
        let query = format!(
            "SELECT add_compression_policy($1, INTERVAL '{}');",
            compress_after.to_postgres_interval()
        );

        diesel::sql_query(query)
            .bind::<Text, _>(Self::TABLE_NAME)
            .execute(conn)?;
        Ok(())
    }

    /// Add a compression policy from a string interval.
    fn add_compression_policy_str(
        conn: &mut PgConnection,
        compress_after: &str,
    ) -> QueryResult<()> {
        let interval = TimeInterval::from_string(compress_after).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        Self::add_compression_policy(conn, interval)
    }

    /// Add a retention policy to automatically drop old data.
    fn add_retention_policy(conn: &mut PgConnection, drop_after: TimeInterval) -> QueryResult<()> {
        let query = format!(
            "SELECT add_retention_policy($1, INTERVAL '{}');",
            drop_after.to_postgres_interval()
        );

        diesel::sql_query(query)
            .bind::<Text, _>(Self::TABLE_NAME)
            .execute(conn)?;
        Ok(())
    }

    /// Add a retention policy from a string interval.
    fn add_retention_policy_str(conn: &mut PgConnection, drop_after: &str) -> QueryResult<()> {
        let interval = TimeInterval::from_string(drop_after).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        Self::add_retention_policy(conn, interval)
    }
}

/// Macro to implement the Hypertable trait for a table.
#[macro_export]
macro_rules! hypertable {
    ($table_name:ident, $time_column:ident) => {
        impl $crate::schema::Hypertable for $table_name::table {
            const TABLE_NAME: &'static str = stringify!($table_name);
            const TIME_COLUMN: &'static str = stringify!($time_column);
        }
    };
}

/// Configuration for continuous aggregates.
#[derive(Debug, Clone)]
pub struct ContinuousAggregateConfig {
    pub view_name: String,
    pub query: String,
    pub refresh_lag: Option<String>,
    pub refresh_interval: Option<String>,
}

impl ContinuousAggregateConfig {
    /// Create a new continuous aggregate configuration.
    pub fn new(view_name: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            view_name: view_name.into(),
            query: query.into(),
            refresh_lag: None,
            refresh_interval: None,
        }
    }

    /// Set the refresh lag for the continuous aggregate.
    pub fn with_refresh_lag(mut self, lag: impl Into<String>) -> Self {
        self.refresh_lag = Some(lag.into());
        self
    }

    /// Set the refresh interval for the continuous aggregate.
    pub fn with_refresh_interval(mut self, interval: impl Into<String>) -> Self {
        self.refresh_interval = Some(interval.into());
        self
    }

    /// Create the continuous aggregate.
    pub fn create(&self, conn: &mut PgConnection) -> QueryResult<()> {
        // Validate the view name
        let view_identifier = SqlIdentifier::new(&self.view_name).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        // Note: We cannot parameterize the view name or query in CREATE MATERIALIZED VIEW
        // because PostgreSQL doesn't support it. However, we validate the view name.
        // The query parameter should be validated by the caller.
        let create_sql = format!(
            "CREATE MATERIALIZED VIEW {} WITH (timescaledb.continuous) AS {};",
            view_identifier.escaped(),
            self.query
        );

        diesel::sql_query(create_sql).execute(conn)?;

        // Add refresh policy if specified
        if let (Some(interval_str), lag_opt) = (&self.refresh_interval, &self.refresh_lag) {
            let interval = TimeInterval::from_string(interval_str).map_err(|e| {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::Unknown,
                    Box::new(e.to_string()),
                )
            })?;

            let mut refresh_sql = format!(
                "SELECT add_continuous_aggregate_policy($1, start_offset => NULL, end_offset => INTERVAL '{}'",
                interval.to_postgres_interval()
            );

            if let Some(lag_str) = lag_opt {
                let lag = TimeInterval::from_string(lag_str).map_err(|e| {
                    diesel::result::Error::DatabaseError(
                        diesel::result::DatabaseErrorKind::Unknown,
                        Box::new(e.to_string()),
                    )
                })?;
                refresh_sql.push_str(&format!(
                    ", schedule_interval => INTERVAL '{}'",
                    lag.to_postgres_interval()
                ));
            }

            refresh_sql.push_str(");");
            diesel::sql_query(refresh_sql)
                .bind::<Text, _>(&self.view_name)
                .execute(conn)?;
        }

        Ok(())
    }
}

/// Module for managing TimescaleDB chunks.
pub mod chunks {
    use super::*;

    /// Information about a chunk in a hypertable.
    #[derive(Debug, Clone, QueryableByName)]
    pub struct ChunkInfo {
        #[diesel(sql_type = Text)]
        pub chunk_schema: String,
        #[diesel(sql_type = Text)]
        pub chunk_name: String,
        #[diesel(sql_type = Text)]
        pub table_name: String,
        #[diesel(sql_type = Nullable<Timestamptz>)]
        pub range_start: Option<chrono::DateTime<chrono::Utc>>,
        #[diesel(sql_type = Nullable<Timestamptz>)]
        pub range_end: Option<chrono::DateTime<chrono::Utc>>,
    }

    /// Get information about chunks for a hypertable.
    pub fn get_chunk_info(
        conn: &mut PgConnection,
        table_name: &str,
    ) -> QueryResult<Vec<ChunkInfo>> {
        // Validate table name
        let _table_identifier = SqlIdentifier::new(table_name).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        diesel::sql_query(
            "SELECT chunk_schema, chunk_name, table_name, range_start, range_end 
             FROM timescaledb_information.chunks 
             WHERE hypertable_name = $1",
        )
        .bind::<Text, _>(table_name)
        .load::<ChunkInfo>(conn)
    }

    /// Drop chunks older than a specified time.
    pub fn drop_old_chunks(
        conn: &mut PgConnection,
        table_name: &str,
        older_than: chrono::DateTime<chrono::Utc>,
    ) -> QueryResult<()> {
        // Validate table name
        let _table_identifier = SqlIdentifier::new(table_name).map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        diesel::sql_query("SELECT drop_chunks($1, $2);")
            .bind::<Text, _>(table_name)
            .bind::<Timestamptz, _>(older_than)
            .execute(conn)?;
        Ok(())
    }
}
