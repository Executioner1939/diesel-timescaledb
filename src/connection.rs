//! Connection utilities for TimescaleDB with Diesel.

use diesel::pg::PgConnection;
use diesel::prelude::*;

/// A wrapper around `PgConnection` with TimescaleDB-specific functionality.
pub struct TimescaleDbConnection {
    connection: PgConnection,
}

impl TimescaleDbConnection {
    /// Create a new TimescaleDB connection from a PostgreSQL connection.
    pub fn new(connection: PgConnection) -> Self {
        Self { connection }
    }

    /// Establish a connection to TimescaleDB using the given database URL.
    pub fn establish(database_url: &str) -> ConnectionResult<Self> {
        let connection = PgConnection::establish(database_url)?;
        Ok(Self::new(connection))
    }

    /// Get a reference to the underlying PostgreSQL connection.
    pub fn connection(&self) -> &PgConnection {
        &self.connection
    }

    /// Get a mutable reference to the underlying PostgreSQL connection.
    pub fn connection_mut(&mut self) -> &mut PgConnection {
        &mut self.connection
    }
}

impl std::ops::Deref for TimescaleDbConnection {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl std::ops::DerefMut for TimescaleDbConnection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}
