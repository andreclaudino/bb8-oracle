#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Oracle support for the bb8 connection pool.
//!
//! If you want to use chrono data types, enable the ```chrono``` feature:
//!```toml
//![dependencies]
//!bb8-oracle = { version = "0.2.0", features = ["chrono"] }
//!```

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
pub use bb8;
pub use oracle;

/// A `bb8::ManageConnection` for `oracle::Connection`s.
///
/// # Example
/// ```no_run
/// use std::thread;
/// use bb8_oracle::OracleConnectionManager;
///
/// let manager = OracleConnectionManager::new("user", "password", "localhost");
/// let pool = bb8::Pool::builder()
///      .max_size(15)
///      .build(manager).await
///      .unwrap();
///
/// for _ in 0..20 {
///     let pool = pool.clone();
///     thread::spawn(move || {
///         let conn = pool.get().unwrap();
///         // use the connection
///         // it will be returned to the pool when it falls out of scope.
///     });
/// }
/// ```
#[derive(Debug)]
pub struct OracleConnectionManager {
    connector: oracle::Connector,
}

impl OracleConnectionManager {
    /// Initialise the connection manager with the data needed to create new connections.
    /// Refer to the documentation of `oracle::Connection` for further details on the parameters.
    ///
    /// # Example
    /// ```
    /// # use bb8_oracle::OracleConnectionManager;
    /// let manager = OracleConnectionManager::new("user", "password", "localhost");
    /// ```
    pub fn new<U: Into<String>, P: Into<String>, C: Into<String>>(username: U, password: P, connect_string: C) -> OracleConnectionManager {
        let connector = oracle::Connector::new(username, password, connect_string);
        OracleConnectionManager {
            connector,
        }
    }

    /// Initialise the connection manager with the data needed to create new connections using `oracle::Connector`.
    /// This allows setting additional connection data.
    ///
    /// If a connection can be established only with a username, password and connect string, use `new` instead.
    ///
    /// # Example
    /// ```
    /// # use bb8_oracle::OracleConnectionManager;
    /// // connect system/manager as sysdba
    /// let mut connector = oracle::Connector::new("system", "manager", "");
    /// connector.privilege(oracle::Privilege::Sysdba);
    /// let manager = OracleConnectionManager::from_connector(connector);
    /// ```
    pub fn from_connector(connector: oracle::Connector) -> OracleConnectionManager {
        OracleConnectionManager { connector }
    }
}

/// An error that can occur during Oracle database connection pool operations.
#[derive(Debug)]
pub enum Error {
    /// An error that occurred during database communication.
    Database(oracle::Error),

    /// An error that occurred because a pool operation panicked.
    Panic(tokio::task::JoinError),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(e) => write!(f, "database error: {}", e),
            Self::Panic(e) => write!(f, "operation panicked: {}", e),
        }
    }
}
impl std::error::Error for Error {
}


#[async_trait]
impl bb8::ManageConnection for OracleConnectionManager {
    type Connection = Arc<oracle::Connection>;
    type Error = Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let connector_clone = self.connector.clone();
        let result = tokio::task::spawn_blocking(move || {
            connector_clone.connect()
        }).await;
        match result {
            Ok(Ok(c)) => Ok(Arc::new(c)),
            Ok(Err(e)) => Err(Error::Database(e)),
            Err(e) => Err(Error::Panic(e)),
        }
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        let conn_clone = Arc::clone(&conn);
        let result = tokio::task::spawn_blocking(move || {
            conn_clone.ping()
        }).await;
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(Error::Database(e)),
            Err(e) => Err(Error::Panic(e)),
        }
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        !matches!(conn.status(), Ok(oracle::ConnStatus::Normal))
    }
}
