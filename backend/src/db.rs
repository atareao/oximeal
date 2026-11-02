//! Gestión del pool de conexiones a SQLite y migraciones.

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

/// Crea el pool de conexiones, asegura el directorio de la BD y aplica migraciones.
///
/// Configura `create_if_missing`, `journal_mode=WAL`, `foreign_keys=on` y un
/// `busy_timeout` de 5 s, alineado con ADR-7.
pub async fn init_pool(database_url: &str) -> Result<SqlitePool> {
    ensure_parent_dir(database_url)?;

    let options = SqliteConnectOptions::from_str(database_url)
        .with_context(|| format!("URL de base de datos inválida: {database_url}"))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .context("no se pudo conectar a SQLite")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("no se pudieron aplicar las migraciones")?;

    Ok(pool)
}

/// Comprueba que la base de datos responde.
pub async fn ping(pool: &SqlitePool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await
        .context("ping a SQLite falló")?;
    Ok(())
}

/// Garantiza que el directorio padre del fichero SQLite exista.
fn ensure_parent_dir(database_url: &str) -> Result<()> {
    let Some(path) = file_path_from_url(database_url) else {
        return Ok(());
    };

    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("no se pudo crear el directorio {}", parent.display()))?;
    }

    Ok(())
}

/// Extrae la ruta de fichero de una URL SQLite, si aplica.
///
/// Devuelve `None` para bases en memoria (`:memory:`) o URLs sin ruta.
fn file_path_from_url(database_url: &str) -> Option<PathBuf> {
    let rest = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))?;

    // Descartar parámetros de consulta (`?mode=...`).
    let path = rest.split('?').next().unwrap_or(rest);

    if path.is_empty() || path == ":memory:" {
        return None;
    }

    Some(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_relative_file_url() {
        assert_eq!(
            file_path_from_url("sqlite://data/oximeal.db"),
            Some(PathBuf::from("data/oximeal.db"))
        );
    }

    #[test]
    fn parses_absolute_file_url() {
        assert_eq!(
            file_path_from_url("sqlite:///var/lib/oximeal.db"),
            Some(PathBuf::from("/var/lib/oximeal.db"))
        );
    }

    #[test]
    fn ignores_memory_url() {
        assert_eq!(file_path_from_url("sqlite::memory:"), None);
        assert_eq!(file_path_from_url("sqlite://:memory:"), None);
    }

    #[test]
    fn strips_query_parameters() {
        assert_eq!(
            file_path_from_url("sqlite://data/oximeal.db?mode=rwc"),
            Some(PathBuf::from("data/oximeal.db"))
        );
    }
}
