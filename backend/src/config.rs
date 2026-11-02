//! Configuración de la aplicación, cargada desde variables de entorno.
//!
//! Se apoya en `dotenvy` para leer un fichero `.env` si existe (su ausencia
//! no es un error). Cada variable tiene un valor por defecto sensato para que
//! el binario arranque en un entorno self-hosted sin configuración previa.

use std::env;

const DEFAULT_DATABASE_URL: &str = "sqlite://data/oximeal.db";
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

/// Configuración inmutable del proceso.
#[derive(Clone, Debug)]
pub struct Config {
    /// URL de conexión de SQLite, p. ej. `sqlite://data/oximeal.db`.
    pub database_url: String,
    /// Dirección de escucha HTTP, p. ej. `0.0.0.0:8080`.
    pub bind_addr: String,
}

impl Config {
    /// Carga la configuración desde el entorno, aplicando los valores por defecto.
    ///
    /// Lee el fichero `.env` (si existe) antes de resolver las variables.
    pub fn from_env() -> Self {
        // La ausencia de `.env` no es un error.
        let _ = dotenvy::dotenv();

        Self {
            database_url: env_or(DEFAULT_DATABASE_URL, "DATABASE_URL"),
            bind_addr: env_or(DEFAULT_BIND_ADDR, "BIND_ADDR"),
        }
    }
}

/// Devuelve el valor de `key` si está definido y no está vacío; si no, `default`.
fn env_or(default: &str, key: &str) -> String {
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_or_falls_back_to_default() {
        assert_eq!(env_or("fallback", "OXIMEAL_TEST_MISSING_VAR"), "fallback");
    }

    #[test]
    fn env_or_uses_present_value() {
        // SAFETY: el nombre de variable es único para este test.
        unsafe { env::set_var("OXIMEAL_TEST_PRESENT_VAR", "value") };
        assert_eq!(env_or("fallback", "OXIMEAL_TEST_PRESENT_VAR"), "value");
        unsafe { env::remove_var("OXIMEAL_TEST_PRESENT_VAR") };
    }
}
