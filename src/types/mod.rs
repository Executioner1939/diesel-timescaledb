//! TimescaleDB-specific types and type mappings for Diesel.

use chrono::{DateTime, Utc};
use diesel::deserialize::{self, FromSql};
use diesel::pg::Pg;
use diesel::serialize::{self, Output, ToSql};
use diesel::sql_types::*;

/// A timestamp with timezone type optimized for time-series data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimestampTz(pub DateTime<Utc>);

impl TimestampTz {
    pub fn new(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn inner(&self) -> &DateTime<Utc> {
        &self.0
    }
}

impl From<DateTime<Utc>> for TimestampTz {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<TimestampTz> for DateTime<Utc> {
    fn from(ts: TimestampTz) -> Self {
        ts.0
    }
}

impl ToSql<Timestamptz, Pg> for TimestampTz {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        <DateTime<Utc> as ToSql<Timestamptz, Pg>>::to_sql(&self.0, out)
    }
}

impl FromSql<Timestamptz, Pg> for TimestampTz {
    fn from_sql(bytes: diesel::pg::PgValue<'_>) -> deserialize::Result<Self> {
        let dt = <DateTime<Utc> as FromSql<Timestamptz, Pg>>::from_sql(bytes)?;
        Ok(Self(dt))
    }
}

/// Trait for types that can be used as time dimensions in TimescaleDB.
pub trait TimeDimension {
    /// The SQL type of this time dimension.
    type SqlType;
    
    /// Convert to a value suitable for database operations.
    fn to_sql_value(&self) -> Self::SqlType;
}

impl TimeDimension for TimestampTz {
    type SqlType = DateTime<Utc>;
    
    fn to_sql_value(&self) -> Self::SqlType {
        self.0
    }
}

impl TimeDimension for DateTime<Utc> {
    type SqlType = DateTime<Utc>;
    
    fn to_sql_value(&self) -> Self::SqlType {
        *self
    }
}