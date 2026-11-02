//! Modelo de sesión server-side y su almacén sobre SQLite.
//!
//! El navegador solo recibe una cookie opaca con el identificador de sesión;
//! todo el estado (incluidos los tokens) vive en la tabla `sessions` de SQLite.
//! Este módulo define:
//!
//! - [`SessionState`]: contenido tipado guardado bajo la clave [`SESSION_KEY`].
//! - [`SqliteSessionStore`]: implementación de `tower_sessions::SessionStore`
//!   y `ExpiredDeletion` sobre el pool de `sqlx`.

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha512};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tower_sessions::{ExpiredDeletion, SessionStore, session::Record};

/// Clave bajo la que se guarda [`SessionState`] dentro de la sesión.
pub const SESSION_KEY: &str = "state";

/// Deriva la clave de firma de la cookie a partir del secreto de sesión.
///
/// Se normaliza el secreto con SHA-512 (64 bytes) para que
/// `cookie::Key::from` no pueda fallar por longitud, independientemente de lo
/// corto o largo que sea `SESSION_SECRET`.
pub fn cookie_key(secret: &str) -> tower_sessions::cookie::Key {
    let digest = Sha512::digest(secret.as_bytes());
    tower_sessions::cookie::Key::from(digest.as_ref())
}

/// Estado lógico guardado en la sesión server-side.
///
/// Contiene el flujo pre-autenticación (mientras se completa el login) y la
/// sesión autenticada una vez validado el callback. Nunca sale al navegador.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionState {
    /// Flujo OIDC en curso (`state`, `nonce`, PKCE). `None` si no hay login pendiente.
    #[serde(default)]
    pub flow: Option<PreAuthFlow>,
    /// Sesión autenticada. `None` si el usuario aún no ha completado el login.
    #[serde(default)]
    pub auth: Option<AuthenticatedSession>,
}

/// Datos de la petición de autorización que deben sobrevivir entre
/// `/auth/login` y `/auth/callback`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreAuthFlow {
    /// Valor `state` enviado al proveedor y devuelto en el callback.
    pub state: String,
    /// Valor `nonce` ligado al ID token.
    pub nonce: String,
    /// Verificador PKCE (`code_verifier`); el `code_challenge` se deriva de él.
    pub code_verifier: String,
    /// Ruta interna a la que volver tras el login (solo rutas locales).
    #[serde(default)]
    pub return_to: Option<String>,
}

/// Sesión autenticada de un usuario.
///
/// Los tokens se guardan aquí (server-side) y **nunca** se serializan a la
/// respuesta de `/auth/me`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedSession {
    /// Claim `sub` del ID token.
    pub subject: String,
    /// Nombre para mostrar (claim `name`).
    #[serde(default)]
    pub name: Option<String>,
    /// Correo (claim `email`).
    #[serde(default)]
    pub email: Option<String>,
    /// Grupos del usuario (claim `groups`).
    #[serde(default)]
    pub groups: Vec<String>,
    /// Permisos concedidos, derivados del claim `scope` del access token.
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Token anti-CSRF para mutaciones autenticadas por cookie.
    pub csrf_token: String,
    /// Access token (RFC 9068). Solo server-side.
    pub access_token: String,
    /// Refresh token (`offline_access`). Solo server-side.
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// ID token original. Solo server-side.
    #[serde(default)]
    pub id_token: Option<String>,
    /// Instante Unix (segundos) en el que caduca el access token.
    pub access_token_expires_at: i64,
}

/// Almacén de sesiones de `tower-sessions` sobre la tabla `sessions` de SQLite.
///
/// Serializa `Record.data` (un mapa JSON) a un BLOB con `serde_json` y guarda
/// la caducidad como segundos Unix. Implementa [`ExpiredDeletion`] para que la
/// limpieza periódica pueda lanzarse como tarea en segundo plano.
#[derive(Debug, Clone)]
pub struct SqliteSessionStore {
    pool: SqlitePool,
}

impl SqliteSessionStore {
    /// Crea el almacén sobre el pool dado.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Pool subyacente (para tareas de limpieza u otros usos).
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Serializa el mapa de datos del registro.
    fn encode(
        data: &HashMap<String, Value>,
    ) -> Result<Vec<u8>, tower_sessions::session_store::Error> {
        serde_json::to_vec(data)
            .map_err(|error| tower_sessions::session_store::Error::Encode(error.to_string()))
    }

    /// Deserializa el BLOB a un mapa de datos.
    fn decode(
        bytes: &[u8],
    ) -> Result<HashMap<String, Value>, tower_sessions::session_store::Error> {
        serde_json::from_slice(bytes)
            .map_err(|error| tower_sessions::session_store::Error::Decode(error.to_string()))
    }

    /// Inserta o actualiza el registro devolviendo el error crudo de `sqlx`.
    async fn upsert(
        &self,
        id: tower_sessions::session::Id,
        data: Vec<u8>,
        expiry: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO sessions (id, data, expiry) VALUES (?1, ?2, ?3) \
             ON CONFLICT(id) DO UPDATE SET data = excluded.data, expiry = excluded.expiry",
        )
        .bind(id.to_string())
        .bind(data)
        .bind(expiry)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

/// Indica si un error de `sqlx` corresponde a una violación de unicidad.
fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db) if db.is_unique_violation())
}

#[async_trait]
impl SessionStore for SqliteSessionStore {
    async fn create(&self, record: &mut Record) -> tower_sessions::session_store::Result<()> {
        // Mitigación de colisiones de id: ante una violación de unicidad, se
        // genera un id nuevo (el identificador por defecto ya es aleatorio).
        let data = Self::encode(&record.data)?;
        let expiry = record.expiry_date.unix_timestamp();
        for _ in 0..5 {
            match self.upsert(record.id, data.clone(), expiry).await {
                Ok(()) => return Ok(()),
                Err(error) if is_unique_violation(&error) => {
                    record.id = tower_sessions::session::Id::default();
                }
                Err(error) => {
                    return Err(tower_sessions::session_store::Error::Backend(
                        error.to_string(),
                    ));
                }
            }
        }
        Err(tower_sessions::session_store::Error::Backend(
            "no se pudo crear la sesión: colisión de identificador".to_string(),
        ))
    }

    async fn save(&self, record: &Record) -> tower_sessions::session_store::Result<()> {
        let data = Self::encode(&record.data)?;
        self.upsert(record.id, data, record.expiry_date.unix_timestamp())
            .await
            .map_err(|error| tower_sessions::session_store::Error::Backend(error.to_string()))?;
        Ok(())
    }

    async fn load(
        &self,
        session_id: &tower_sessions::session::Id,
    ) -> tower_sessions::session_store::Result<Option<Record>> {
        let row: Option<(Vec<u8>, i64)> =
            sqlx::query_as("SELECT data, expiry FROM sessions WHERE id = ?1")
                .bind(session_id.to_string())
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| {
                    tower_sessions::session_store::Error::Backend(error.to_string())
                })?;

        let Some((data, expiry)) = row else {
            return Ok(None);
        };

        let expiry_date = OffsetDateTime::from_unix_timestamp(expiry)
            .map_err(|error| tower_sessions::session_store::Error::Decode(error.to_string()))?;

        if expiry_date <= OffsetDateTime::now_utc() {
            return Ok(None);
        }

        Ok(Some(Record {
            id: *session_id,
            data: Self::decode(&data)?,
            expiry_date,
        }))
    }

    async fn delete(
        &self,
        session_id: &tower_sessions::session::Id,
    ) -> tower_sessions::session_store::Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?1")
            .bind(session_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|error| tower_sessions::session_store::Error::Backend(error.to_string()))?;
        Ok(())
    }
}

#[async_trait]
impl ExpiredDeletion for SqliteSessionStore {
    async fn delete_expired(&self) -> tower_sessions::session_store::Result<()> {
        sqlx::query("DELETE FROM sessions WHERE expiry < ?1")
            .bind(OffsetDateTime::now_utc().unix_timestamp())
            .execute(&self.pool)
            .await
            .map_err(|error| tower_sessions::session_store::Error::Backend(error.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    async fn store() -> SqliteSessionStore {
        let dir = tempfile::tempdir().expect("tempdir");
        let url = format!("sqlite://{}", dir.path().join("s.db").display());
        let pool = db::init_pool(&url).await.expect("pool");
        std::mem::forget(dir);
        SqliteSessionStore::new(pool)
    }

    fn record(state: SessionState, expiry_date: OffsetDateTime) -> Record {
        let mut data = HashMap::new();
        data.insert(
            SESSION_KEY.to_string(),
            serde_json::to_value(&state).unwrap(),
        );
        Record {
            id: tower_sessions::session::Id::default(),
            data,
            expiry_date,
        }
    }

    #[tokio::test]
    async fn save_then_load_round_trips_state() {
        let store = store().await;
        let mut rec = record(
            SessionState::default(),
            OffsetDateTime::now_utc() + time::Duration::hours(1),
        );
        store.create(&mut rec).await.expect("create");

        let loaded = store.load(&rec.id).await.expect("load").expect("present");
        let state: SessionState =
            serde_json::from_value(loaded.data.get(SESSION_KEY).cloned().unwrap()).unwrap();
        assert!(state.auth.is_none());
    }

    #[tokio::test]
    async fn expired_record_is_not_returned() {
        let store = store().await;
        let rec = record(
            SessionState::default(),
            OffsetDateTime::now_utc() - time::Duration::hours(1),
        );
        store.save(&rec).await.expect("save");
        assert!(store.load(&rec.id).await.expect("load").is_none());
    }

    #[tokio::test]
    async fn delete_removes_record() {
        let store = store().await;
        let mut rec = record(
            SessionState::default(),
            OffsetDateTime::now_utc() + time::Duration::hours(1),
        );
        store.create(&mut rec).await.expect("create");
        store.delete(&rec.id).await.expect("delete");
        assert!(store.load(&rec.id).await.expect("load").is_none());
    }

    #[tokio::test]
    async fn delete_expired_removes_stale_rows() {
        let store = store().await;
        let stale = record(
            SessionState::default(),
            OffsetDateTime::now_utc() - time::Duration::hours(1),
        );
        store.save(&stale).await.expect("save stale");
        store.delete_expired().await.expect("delete_expired");
        assert!(store.load(&stale.id).await.expect("load").is_none());
    }
}
