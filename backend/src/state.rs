//! Estado compartido de la aplicación, inyectado en los handlers de Axum.

use std::sync::Arc;

use sqlx::SqlitePool;

use crate::config::Config;

/// Estado clonable que contiene el pool de base de datos y la configuración.
#[derive(Clone)]
pub struct AppState {
    /// Pool de conexiones a SQLite.
    pub pool: SqlitePool,
    /// Configuración de la aplicación (compartida con `Arc`).
    pub config: Arc<Config>,
}
