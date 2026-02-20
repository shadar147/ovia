use ovia_common::error::{OviaError, OviaResult};
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: String,
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

impl AppConfig {
    /// Load configuration from environment variables.
    /// Loads `.env` file if present, then reads required vars.
    pub fn from_env() -> OviaResult<Self> {
        // Best-effort .env load; ignore if missing
        let _ = dotenvy::dotenv();

        Ok(Self {
            database_url: get_var("DATABASE_URL")?,
            redis_url: get_var_or("REDIS_URL", "redis://127.0.0.1:6379"),
            host: get_var_or("HOST", "0.0.0.0"),
            port: get_var_or("PORT", "8080")
                .parse()
                .map_err(|e| OviaError::Config(format!("invalid PORT: {e}")))?,
            log_level: get_var_or("LOG_LEVEL", "info"),
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn get_var(key: &str) -> OviaResult<String> {
    env::var(key).map_err(|_| OviaError::Config(format!("{key} is required but not set")))
}

fn get_var_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn config_from_env_succeeds_with_required_vars() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");

        // Set required vars for this test
        env::set_var("DATABASE_URL", "postgres://localhost/ovia_test");

        let cfg = AppConfig::from_env().expect("should parse config");
        assert_eq!(cfg.database_url, "postgres://localhost/ovia_test");
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.log_level, "info");

        env::remove_var("DATABASE_URL");
    }

    #[test]
    fn config_from_env_fails_without_database_url() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");

        env::remove_var("DATABASE_URL");
        let result = AppConfig::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn bind_addr_formats_correctly() {
        let cfg = AppConfig {
            database_url: String::new(),
            redis_url: String::new(),
            host: "127.0.0.1".to_owned(),
            port: 3000,
            log_level: "debug".to_owned(),
        };
        assert_eq!(cfg.bind_addr(), "127.0.0.1:3000");
    }
}
