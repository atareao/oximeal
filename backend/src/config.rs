//! Configuración de la aplicación, cargada desde variables de entorno.
//!
//! Se apoya en `dotenvy` para leer un fichero `.env` si existe (su ausencia
//! no es un error). Cada variable tiene un valor por defecto sensato para que
//! el binario arranque en un entorno self-hosted sin configuración previa.
//!
//! La autenticación OIDC se activa con `OIDC_ENABLED=true`. En ese caso la
//! configuración pasa a ser obligatoria y [`Config::from_env`] falla si falta
//! algún valor o si el secreto de sesión es vacío o un valor por defecto
//! conocido. Con `OIDC_ENABLED=false` la API queda abierta (solo desarrollo).

use std::env;
use std::time::Duration;

use anyhow::{Result, bail};

const DEFAULT_DATABASE_URL: &str = "sqlite://data/oximeal.db";
const DEFAULT_BIND_ADDR: &str = "0.0.0.0:8080";

const DEFAULT_SESSION_TTL_SECS: u64 = 2_592_000; // 30 días
const DEFAULT_SESSION_REFRESH_MARGIN_SECS: u64 = 300; // 5 minutos
const DEFAULT_SESSION_ABSOLUTE_TTL_SECS: u64 = 43_200; // 12 horas
const DEFAULT_SESSION_COOKIE_NAME: &str = "oximeal_session";

/// Secretos de sesión nunca aceptables en producción.
const FORBIDDEN_SESSION_SECRETS: &[&str] =
    &["changeme", "change-me", "secret", "default", "oximeal"];

/// Variables OIDC obligatorias cuando `OIDC_ENABLED=true`.
const REQUIRED_OIDC_VARS: &[&str] = &[
    "OIDC_ISSUER",
    "OIDC_CLIENT_ID",
    "OIDC_CLIENT_SECRET",
    "OIDC_API_AUDIENCE",
    "OIDC_REDIRECT_URI",
    "OIDC_POST_LOGOUT_REDIRECT_URI",
    "OIDC_ALLOWED_GROUPS",
    "SESSION_SECRET",
];

/// Configuración inmutable del proceso.
#[derive(Clone, Debug)]
pub struct Config {
    /// URL de conexión de SQLite, p. ej. `sqlite://data/oximeal.db`.
    pub database_url: String,
    /// Dirección de escucha HTTP, p. ej. `0.0.0.0:8080`.
    pub bind_addr: String,
    /// Orígenes CORS permitidos (`ALLOWED_ORIGINS`, CSV). Vacío ⇒ ninguno.
    pub allowed_origins: Vec<String>,
    /// Hosts exentos del veto de rango privado en la importación de recetas
    /// (`IMPORT_ALLOWED_HOSTS`, CSV). Vacío ⇒ estricto (ningún host exceptuado).
    ///
    /// Un host de esta lista omite **únicamente** la guarda de rango
    /// privado/loopback/link-local; el esquema, las redirecciones, el timeout y
    /// el tope de tamaño se siguen aplicando siempre.
    pub import_allowed_hosts: Vec<String>,
    /// Configuración de sesión server-side.
    pub session: SessionConfig,
    /// Configuración OIDC; `None` ⇔ `OIDC_ENABLED=false`.
    pub oidc: Option<OidcConfig>,
}

/// Parámetros del cliente OIDC (Pocket ID).
#[derive(Clone, Debug)]
pub struct OidcConfig {
    /// Issuer (`iss`) del proveedor de identidad.
    pub issuer: String,
    /// `client_id` del cliente confidencial BFF.
    pub client_id: String,
    /// `client_secret` del cliente confidencial BFF.
    pub client_secret: String,
    /// Audiencia esperada del access token (el *resource* del API).
    pub api_audience: String,
    /// URI de redirección tras el login.
    pub redirect_uri: String,
    /// URI de redirección tras el logout.
    ///
    /// Reservada para el RP-initiated logout (redirección al
    /// `end_session_endpoint` del IdP). Es obligatoria por contrato de
    /// configuración, pero **no** se usa al servir `POST /auth/logout`: este
    /// sigue respondiendo 204 tras destruir la sesión y limpiar la cookie.
    pub post_logout_redirect_uri: String,
    /// Grupos autorizados a iniciar sesión (allowlist).
    pub allowed_groups: Vec<String>,
}

/// Parámetros de la sesión server-side.
#[derive(Clone, Debug)]
pub struct SessionConfig {
    /// Secreto de firma de la cookie. `None` solo si OIDC está desactivado.
    pub secret: Option<String>,
    /// Vida útil de la sesión.
    pub ttl: Duration,
    /// Vida absoluta de la sesión desde el inicio del login. El refresco de
    /// tokens **no** la extiende.
    pub absolute_ttl: Duration,
    /// Margen antes de la caducidad en el que se refresca la sesión.
    pub refresh_margin: Duration,
    /// Nombre de la cookie de sesión.
    pub cookie_name: String,
    /// `Secure` en la cookie (obligatorio fuera de localhost).
    pub cookie_secure: bool,
}

/// Configuración por defecto de sesión, usada cuando OIDC está desactivado.
impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            secret: None,
            ttl: Duration::from_secs(DEFAULT_SESSION_TTL_SECS),
            absolute_ttl: Duration::from_secs(DEFAULT_SESSION_ABSOLUTE_TTL_SECS),
            refresh_margin: Duration::from_secs(DEFAULT_SESSION_REFRESH_MARGIN_SECS),
            cookie_name: DEFAULT_SESSION_COOKIE_NAME.to_string(),
            cookie_secure: true,
        }
    }
}

impl Config {
    /// Carga la configuración desde el entorno, aplicando los valores por defecto.
    ///
    /// Lee el fichero `.env` (si existe) antes de resolver las variables.
    /// Devuelve `Err` si `OIDC_ENABLED=true` y la configuración es incompleta
    /// o el secreto de sesión es un valor por defecto conocido.
    pub fn from_env() -> Result<Self> {
        // La ausencia de `.env` no es un error.
        let _ = dotenvy::dotenv();

        let session = load_session_config();
        let allowed_origins = parse_csv(&env_or_default("ALLOWED_ORIGINS", ""));
        let import_allowed_hosts = parse_csv(&env_or_default("IMPORT_ALLOWED_HOSTS", ""));

        let oidc = if parse_bool(&env_or_default("OIDC_ENABLED", "false")) {
            Some(load_oidc_config(&session)?)
        } else {
            tracing::warn!(
                "OIDC_ENABLED=false: la API queda ABIERTA y sin autenticación. \
                 Úsalo solo en desarrollo y pruebas."
            );
            None
        };

        // Con OIDC activo, una cookie sin `Secure` viaja también por HTTP. Es
        // válido en localhost (desarrollo), pero se avisa de forma explícita.
        if oidc.is_some() && !session.cookie_secure {
            tracing::warn!(
                "SESSION_COOKIE_SECURE=false con OIDC_ENABLED=true: la cookie de sesión \
                 no se restringe a HTTPS. Aceptable solo en localhost; usa HTTPS en producción."
            );
        }

        Ok(Self {
            database_url: env_or(DEFAULT_DATABASE_URL, "DATABASE_URL"),
            bind_addr: env_or(DEFAULT_BIND_ADDR, "BIND_ADDR"),
            allowed_origins,
            import_allowed_hosts,
            session,
            oidc,
        })
    }

    /// Construye una [`Config`] válida para tests: sin OIDC (API abierta).
    ///
    /// Evita repetir la construcción literal del struct en los tests.
    pub fn for_tests(database_url: impl Into<String>, bind_addr: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            bind_addr: bind_addr.into(),
            allowed_origins: Vec::new(),
            // Loopback exento para que los tests de import puedan apuntar a
            // un servidor `wiremock` local.
            import_allowed_hosts: vec!["127.0.0.1".to_string(), "localhost".to_string()],
            session: SessionConfig::default(),
            oidc: None,
        }
    }
}

/// Resuelve la configuración de sesión desde el entorno con sus defaults.
fn load_session_config() -> SessionConfig {
    let ttl = parse_secs("SESSION_TTL_SECS").unwrap_or(DEFAULT_SESSION_TTL_SECS);
    let refresh_margin =
        parse_secs("SESSION_REFRESH_MARGIN_SECS").unwrap_or(DEFAULT_SESSION_REFRESH_MARGIN_SECS);
    let cookie_name = env_or(DEFAULT_SESSION_COOKIE_NAME, "SESSION_COOKIE_NAME");
    let cookie_secure = parse_bool(&env_or_default("SESSION_COOKIE_SECURE", "true"));
    let secret = env::var("SESSION_SECRET")
        .ok()
        .filter(|value| !value.trim().is_empty());

    SessionConfig {
        secret,
        ttl: Duration::from_secs(ttl),
        // Andamiaje RED: aún no se lee `SESSION_ABSOLUTE_TTL_SECS`.
        absolute_ttl: Duration::from_secs(DEFAULT_SESSION_ABSOLUTE_TTL_SECS),
        refresh_margin: Duration::from_secs(refresh_margin),
        cookie_name,
        cookie_secure,
    }
}

/// Valida y construye la configuración OIDC; falla si falta o es insegura.
fn load_oidc_config(session: &SessionConfig) -> Result<OidcConfig> {
    let missing: Vec<&str> = REQUIRED_OIDC_VARS
        .iter()
        .copied()
        .filter(|var| env::var(var).map(|v| v.trim().is_empty()).unwrap_or(true))
        .collect();

    if !missing.is_empty() {
        bail!(
            "falta configuración OIDC obligatoria (OIDC_ENABLED=true): {}",
            missing.join(", ")
        );
    }

    // El secreto de sesión nunca puede ser un valor por defecto conocido.
    if let Some(secret) = &session.secret {
        if FORBIDDEN_SESSION_SECRETS
            .iter()
            .any(|forbidden| secret.eq_ignore_ascii_case(forbidden))
        {
            bail!("SESSION_SECRET usa un valor por defecto conocido; define uno único y aleatorio");
        }
    } else {
        bail!("falta SESSION_SECRET (obligatorio con OIDC_ENABLED=true)");
    }

    Ok(OidcConfig {
        issuer: env_required("OIDC_ISSUER"),
        client_id: env_required("OIDC_CLIENT_ID"),
        client_secret: env_required("OIDC_CLIENT_SECRET"),
        api_audience: env_required("OIDC_API_AUDIENCE"),
        redirect_uri: env_required("OIDC_REDIRECT_URI"),
        post_logout_redirect_uri: env_required("OIDC_POST_LOGOUT_REDIRECT_URI"),
        allowed_groups: parse_csv(&env_required("OIDC_ALLOWED_GROUPS")),
    })
}

/// Devuelve el valor de `key` si está definido y no está vacío; si no, `default`.
fn env_or(default: &str, key: &str) -> String {
    env_or_default(key, default)
}

/// Devuelve el valor de `key` si está definido y no está vacío; si no, `default`.
fn env_or_default(key: &str, default: &str) -> String {
    env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

/// Igual que [`env_or`] pero asume que la variable ya fue validada como presente.
fn env_required(key: &str) -> String {
    env::var(key).unwrap_or_default()
}

/// Parsea un booleano tolerante: `true/1/yes/on` y `false/0/no/off`.
fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    )
}

/// Parsea un entero de segundos; `None` si la variable no es válida.
fn parse_secs(key: &str) -> Option<u64> {
    env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
}

/// Divide una lista CSV, recortando espacios y descartando entradas vacías.
fn parse_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(String::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::Duration;

    /// Serializa los tests que manipulan variables de entorno del proceso.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Variables de entorno gestionadas por la configuración OIDC/sesión.
    const MANAGED_VARS: &[&str] = &[
        "OIDC_ENABLED",
        "OIDC_ISSUER",
        "OIDC_CLIENT_ID",
        "OIDC_CLIENT_SECRET",
        "OIDC_API_AUDIENCE",
        "OIDC_REDIRECT_URI",
        "OIDC_POST_LOGOUT_REDIRECT_URI",
        "OIDC_ALLOWED_GROUPS",
        "ALLOWED_ORIGINS",
        "IMPORT_ALLOWED_HOSTS",
        "SESSION_SECRET",
        "SESSION_TTL_SECS",
        "SESSION_REFRESH_MARGIN_SECS",
        "SESSION_COOKIE_NAME",
        "SESSION_COOKIE_SECURE",
        "ALLOW_INSECURE_COOKIE",
        "SESSION_ABSOLUTE_TTL_SECS",
    ];

    /// Configuración OIDC completa y válida reutilizable por los tests.
    const REQUIRED_OIDC: &[(&str, &str)] = &[
        ("OIDC_ENABLED", "true"),
        ("OIDC_ISSUER", "https://idp.example/"),
        ("OIDC_CLIENT_ID", "oximeal-spa"),
        ("OIDC_CLIENT_SECRET", "s3cr3t-value"),
        ("OIDC_API_AUDIENCE", "https://oximeal.local/api"),
        ("OIDC_REDIRECT_URI", "https://app.example/auth/callback"),
        ("OIDC_POST_LOGOUT_REDIRECT_URI", "https://app.example/"),
        ("OIDC_ALLOWED_GROUPS", "oximeal-users"),
        ("SESSION_SECRET", "a-very-strong-secret-value"),
    ];

    /// Guarda y restaura las variables gestionadas; evita carreras entre tests.
    struct EnvScope {
        _guard: std::sync::MutexGuard<'static, ()>,
        saved: Vec<(&'static str, Option<String>)>,
    }

    impl EnvScope {
        fn new(vars: &[(&str, &str)]) -> Self {
            let guard = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
            let saved: Vec<_> = MANAGED_VARS
                .iter()
                .map(|key| (*key, env::var(key).ok()))
                .collect();
            for key in MANAGED_VARS {
                // SAFETY: acceso serializado por `ENV_LOCK`.
                unsafe { env::remove_var(key) };
            }
            for (key, value) in vars {
                // SAFETY: acceso serializado por `ENV_LOCK`.
                unsafe { env::set_var(key, value) };
            }
            Self {
                _guard: guard,
                saved,
            }
        }
    }

    impl Drop for EnvScope {
        fn drop(&mut self) {
            for (key, value) in &self.saved {
                match value {
                    // SAFETY: acceso serializado por `ENV_LOCK`.
                    Some(value) => unsafe { env::set_var(key, value) },
                    // SAFETY: acceso serializado por `ENV_LOCK`.
                    None => unsafe { env::remove_var(key) },
                }
            }
        }
    }

    /// Capa de `tracing` que acumula el texto de los eventos emitidos.
    #[derive(Clone, Default)]
    struct CaptureLayer {
        lines: std::sync::Arc<Mutex<Vec<String>>>,
    }

    impl<S: tracing::Subscriber> tracing_subscriber::layer::Layer<S> for CaptureLayer {
        fn on_event(
            &self,
            event: &tracing::Event<'_>,
            _ctx: tracing_subscriber::layer::Context<'_, S>,
        ) {
            let mut visitor = MessageVisitor::default();
            event.record(&mut visitor);
            if let Some(message) = visitor.message {
                self.lines
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .push(message);
            }
        }
    }

    /// Visitante que extrae el campo `message` de un evento.
    #[derive(Default)]
    struct MessageVisitor {
        message: Option<String>,
    }

    impl tracing::field::Visit for MessageVisitor {
        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
            if field.name() == "message" {
                self.message = Some(format!("{value:?}"));
            }
        }
    }

    /// Ejecuta `body` con un subscriber de hilo que captura los logs emitidos.
    ///
    /// Se usa `tracing::subscriber::with_default` (subscriber *thread-local*),
    /// por lo que no interfiere con otros tests que corren en paralelo.
    fn capture_logs(body: impl FnOnce()) -> Vec<String> {
        use tracing_subscriber::prelude::*;

        let layer = CaptureLayer::default();
        let lines = layer.lines.clone();
        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, body);
        lines
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
    }

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

    #[test]
    fn oidc_enabled_missing_vars_errors_listing_variables() {
        let _scope = EnvScope::new(&[("OIDC_ENABLED", "true")]);

        let error = Config::from_env().expect_err("la config incompleta debe fallar");
        let message = error.to_string();

        for var in [
            "OIDC_ISSUER",
            "OIDC_CLIENT_ID",
            "OIDC_CLIENT_SECRET",
            "OIDC_API_AUDIENCE",
            "OIDC_REDIRECT_URI",
            "OIDC_POST_LOGOUT_REDIRECT_URI",
            "OIDC_ALLOWED_GROUPS",
            "SESSION_SECRET",
        ] {
            assert!(
                message.contains(var),
                "el mensaje debe listar {var}: {message}"
            );
        }
    }

    #[test]
    fn oidc_enabled_default_session_secret_errors() {
        let _scope = EnvScope::new(REQUIRED_OIDC);
        // SAFETY: acceso serializado por `ENV_LOCK`.
        unsafe { env::set_var("SESSION_SECRET", "changeme") };

        let error = Config::from_env().expect_err("un secreto por defecto debe fallar");
        assert!(
            error.to_string().contains("SESSION_SECRET"),
            "el error debe señalar SESSION_SECRET: {error}"
        );
    }

    #[test]
    fn oidc_disabled_is_ok_without_secret() {
        let _scope = EnvScope::new(&[("OIDC_ENABLED", "false")]);

        let config = Config::from_env().expect("OIDC desactivado no exige secreto");

        assert!(config.oidc.is_none());
        assert!(config.allowed_origins.is_empty());
        assert_eq!(config.session.cookie_name, "__Host-oximeal_session");
        assert_eq!(config.session.ttl, Duration::from_secs(2592000));
        assert_eq!(config.session.absolute_ttl, Duration::from_secs(43200));
        assert_eq!(config.session.refresh_margin, Duration::from_secs(300));
        assert!(config.session.cookie_secure);
        assert!(config.session.secret.is_none());
    }

    #[test]
    fn oidc_enabled_valid_config_parses_fields() {
        let _scope = EnvScope::new(REQUIRED_OIDC);
        // SAFETY: acceso serializado por `ENV_LOCK`.
        unsafe {
            env::set_var(
                "ALLOWED_ORIGINS",
                "https://app.example, https://other.example",
            );
            env::set_var("OIDC_ALLOWED_GROUPS", "oximeal-users, admins");
            env::set_var("SESSION_COOKIE_SECURE", "false");
            env::set_var("ALLOW_INSECURE_COOKIE", "true");
            env::set_var("SESSION_TTL_SECS", "60");
        }

        let config = Config::from_env().expect("config válida");

        let oidc = config.oidc.expect("oidc debe estar presente");
        assert_eq!(oidc.issuer, "https://idp.example/");
        assert_eq!(oidc.client_id, "oximeal-spa");
        assert_eq!(oidc.allowed_groups, vec!["oximeal-users", "admins"]);
        assert_eq!(
            config.allowed_origins,
            vec!["https://app.example", "https://other.example"]
        );
        assert_eq!(config.session.ttl, Duration::from_secs(60));
        assert!(!config.session.cookie_secure);
        assert!(config.session.secret.is_some());
    }

    #[test]
    fn oidc_enabled_empty_groups_errors() {
        let _scope = EnvScope::new(REQUIRED_OIDC);
        // SAFETY: acceso serializado por `ENV_LOCK`.
        unsafe { env::set_var("OIDC_ALLOWED_GROUPS", "") };

        let error = Config::from_env().expect_err("los grupos no pueden ser vacíos");
        assert!(error.to_string().contains("OIDC_ALLOWED_GROUPS"), "{error}");
    }

    #[test]
    fn oidc_disabled_emits_warning() {
        // I1: con OIDC desactivado debe emitirse un warning capturable. Como
        // `main` inicializa el subscriber antes de `Config::from_env`, el aviso
        // llega a los logs de arranque.
        let _scope = EnvScope::new(&[("OIDC_ENABLED", "false")]);

        let lines = capture_logs(|| {
            Config::from_env().expect("config abierta sin OIDC");
        });

        assert!(
            lines.iter().any(|line| line.contains("OIDC_ENABLED=false")),
            "se esperaba el warning de OIDC desactivado; capturado: {lines:?}"
        );
    }

    #[test]
    fn oidc_enabled_insecure_cookie_emits_warning() {
        // M7: cookie sin `Secure` con OIDC activo solo se acepta con opt-in
        // explícito (`ALLOW_INSECURE_COOKIE=true`) y avisa de forma visible.
        let _scope = EnvScope::new(REQUIRED_OIDC);
        // SAFETY: acceso serializado por `ENV_LOCK`.
        unsafe {
            env::set_var("SESSION_COOKIE_SECURE", "false");
            env::set_var("ALLOW_INSECURE_COOKIE", "true");
        }

        let lines = capture_logs(|| {
            Config::from_env().expect("config OIDC válida con opt-in");
        });

        assert!(
            lines
                .iter()
                .any(|line| line.contains("ALLOW_INSECURE_COOKIE")),
            "se esperaba el warning de cookie insegura; capturado: {lines:?}"
        );
    }

    #[test]
    fn import_allowed_hosts_parses_csv_with_default_empty() {
        let _scope = EnvScope::new(&[("OIDC_ENABLED", "false")]);
        let config = Config::from_env().expect("config válida");
        assert!(
            config.import_allowed_hosts.is_empty(),
            "vacío por defecto ⇒ estricto"
        );

        // SAFETY: acceso serializado por `ENV_LOCK`.
        unsafe { env::set_var("IMPORT_ALLOWED_HOSTS", "example.com, 10.0.0.1") };
        let config = Config::from_env().expect("config válida");
        assert_eq!(config.import_allowed_hosts, vec!["example.com", "10.0.0.1"]);
    }

    #[test]
    fn debug_redacts_secrets() {
        // Los secretos nunca deben aparecer en un `Debug` (logs, panics, etc.).
        let config = Config {
            database_url: "sqlite::memory:".to_string(),
            bind_addr: "127.0.0.1:0".to_string(),
            allowed_origins: Vec::new(),
            import_allowed_hosts: Vec::new(),
            session: SessionConfig {
                secret: Some("super-secret-session-value".to_string()),
                ..SessionConfig::default()
            },
            oidc: Some(OidcConfig {
                issuer: "https://idp.example/".to_string(),
                client_id: "oximeal-spa".to_string(),
                client_secret: "super-secret-client-value".to_string(),
                api_audience: "https://oximeal.local/api".to_string(),
                redirect_uri: "https://app.example/auth/callback".to_string(),
                post_logout_redirect_uri: "https://app.example/".to_string(),
                allowed_groups: vec!["oximeal-users".to_string()],
            }),
        };

        let rendered = format!("{config:?}");
        assert!(
            !rendered.contains("super-secret-session-value"),
            "el secreto de sesión no debe aparecer: {rendered}"
        );
        assert!(
            !rendered.contains("super-secret-client-value"),
            "el client_secret no debe aparecer: {rendered}"
        );
        assert!(
            rendered.contains("[REDACTED]"),
            "los secretos deben redactarse de forma explícita: {rendered}"
        );
        // Los campos no secretos siguen visibles para poder depurar.
        assert!(rendered.contains("oximeal-spa"), "{rendered}");
    }

    #[test]
    fn default_cookie_name_uses_host_prefix() {
        assert_eq!(
            SessionConfig::default().cookie_name,
            "__Host-oximeal_session"
        );

        let _scope = EnvScope::new(&[("OIDC_ENABLED", "false")]);
        let config = Config::from_env().expect("config abierta");
        assert_eq!(config.session.cookie_name, "__Host-oximeal_session");
    }

    #[test]
    fn host_prefix_cookie_requires_secure() {
        // El prefijo `__Host-` obliga a `Secure`; sin él la cookie no se emitiría.
        let _scope = EnvScope::new(&[("OIDC_ENABLED", "false")]);
        // SAFETY: acceso serializado por `ENV_LOCK`.
        unsafe {
            env::set_var("SESSION_COOKIE_NAME", "__Host-custom");
            env::set_var("SESSION_COOKIE_SECURE", "false");
        }

        let error = Config::from_env().expect_err("__Host- sin Secure debe fallar");
        assert!(error.to_string().contains("__Host-"), "{error}");
    }

    #[test]
    fn insecure_cookie_with_oidc_fails_closed() {
        // Fail-closed: con OIDC activo, una cookie sin `Secure` no arranca.
        let _scope = EnvScope::new(REQUIRED_OIDC);
        // SAFETY: acceso serializado por `ENV_LOCK`.
        unsafe { env::set_var("SESSION_COOKIE_SECURE", "false") };

        let error = Config::from_env().expect_err("cookie insegura con OIDC debe fallar");
        assert!(
            error.to_string().contains("ALLOW_INSECURE_COOKIE"),
            "el error debe indicar el escape explícito: {error}"
        );
    }

    #[test]
    fn insecure_cookie_allowed_with_explicit_opt_in() {
        let _scope = EnvScope::new(REQUIRED_OIDC);
        // SAFETY: acceso serializado por `ENV_LOCK`.
        unsafe {
            env::set_var("SESSION_COOKIE_SECURE", "false");
            env::set_var("ALLOW_INSECURE_COOKIE", "true");
        }

        let lines = capture_logs(|| {
            let config = Config::from_env().expect("el opt-in explícito permite HTTP");
            assert!(!config.session.cookie_secure);
        });

        assert!(
            lines
                .iter()
                .any(|line| line.contains("ALLOW_INSECURE_COOKIE")),
            "el opt-in debe dejar rastro en los logs: {lines:?}"
        );
    }

    #[test]
    fn absolute_ttl_defaults_to_twelve_hours() {
        let _scope = EnvScope::new(&[("OIDC_ENABLED", "false")]);
        let config = Config::from_env().expect("config abierta");
        assert_eq!(config.session.absolute_ttl, Duration::from_secs(43_200));

        // SAFETY: acceso serializado por `ENV_LOCK`.
        unsafe { env::set_var("SESSION_ABSOLUTE_TTL_SECS", "3600") };
        let config = Config::from_env().expect("config abierta");
        assert_eq!(config.session.absolute_ttl, Duration::from_secs(3_600));
    }

    #[test]
    fn for_tests_builds_open_config() {
        let config = Config::for_tests("sqlite::memory:", "127.0.0.1:0");

        assert_eq!(config.database_url, "sqlite::memory:");
        assert_eq!(config.bind_addr, "127.0.0.1:0");
        assert!(config.oidc.is_none());
        assert!(config.allowed_origins.is_empty());
        // `for_tests` exime loopback para los servidores `wiremock`.
        assert_eq!(config.import_allowed_hosts, vec!["127.0.0.1", "localhost"]);
    }
}
