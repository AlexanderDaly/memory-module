#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use sqlx::AnyPool;

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
use crate::error::{MemoryError, Result};

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("migrations");

#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
/// Run embedded database migrations against the provided connection pool.
///
/// # Errors
/// Returns [`MemoryError::Storage`] if a migration fails.
pub async fn run_migrations(pool: &AnyPool) -> Result<()> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|e| MemoryError::Storage(e.to_string()))
}
