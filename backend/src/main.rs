//! Punto de entrada del binario `oximeal-backend`.

use std::sync::Arc;

use anyhow::Context;
use oximeal_backend::auth::AuthService;
use oximeal_backend::config::Config;
use oximeal_backend::state::AppState;
use oximeal_backend::{app, db};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // El subscriber debe inicializarse ANTES de cargar la configuración: de lo
    // contrario los warnings emitidos durante `Config::from_env` (p. ej. el de
    // `OIDC_ENABLED=false`) se pierden porque nadie los recoge todavía.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env().context("cargar la configuración de entorno")?;

    let pool = db::init_pool(&config.database_url)
        .await
        .context("inicialización de la base de datos")?;

    let config = Arc::new(config);
    let auth = AuthService::build(&config, pool.clone())
        .context("construir el servicio de autenticación")?;
    let state = AppState {
        pool,
        config: Arc::clone(&config),
        auth,
    };
    let router = app(state);

    let listener = tokio::net::TcpListener::bind(&config.bind_addr)
        .await
        .with_context(|| format!("no se pudo enlazar la dirección {}", config.bind_addr))?;

    tracing::info!(
        addr = %config.bind_addr,
        db = %config.database_url,
        "OxiMeal backend escuchando"
    );

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("el servidor HTTP falló")?;

    Ok(())
}

/// Espera a `Ctrl+C` para iniciar un apagado ordenado.
async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %error, "no se pudo escuchar la señal de apagado");
        return;
    }
    tracing::info!("señal de apagado recibida, cerrando el servidor");
}
