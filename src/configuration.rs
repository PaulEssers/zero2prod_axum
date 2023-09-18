use config::{Config, File, FileFormat};
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use sqlx::ConnectOptions;

// Todo: Validate all the settings.

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender: String,
    pub authorization_token: String,
    pub timeout_milliseconds: u64,
}

impl EmailClientSettings {
    pub fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_milliseconds)
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    // Method to join together fields into a single connection string.
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            // Try an encrypted connection, fallback to unencrypted if it fails
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
    // Renamed from `connection_string`
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db()
            .database(&self.database_name)
            .log_statements(log::LevelFilter::Trace)
    }
}

// Read configuration settings from configuration.yaml and return them as a Settings object.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Get the location of the config dir.
    let conf_path = std::env::current_dir()
        .expect("Failed to determine the current directory")
        .join("configuration")
        .into_os_string()
        .into_string()
        .unwrap();

    // Get the paths for the base and environment (local or production) config files to use
    let base_path = format!("{}/{}", conf_path, "base");
    let env_str: String = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "local".into());
    // pass it into enum just to make sure APP_ENVIRONMENT is either local or production.
    let environment = Environment::try_from(env_str).expect("Failed to parse APP_ENVIRONMENT.");
    let env_path = format!("{}/{}", conf_path, environment.as_str());

    // Set the base config
    let builder = Config::builder()
        .add_source(File::new(&base_path, FileFormat::Yaml))
        .add_source(File::new(&env_path, FileFormat::Yaml));

    // For extracting configuration values from environment variables
    // For example APP__APPLICATION_TESTVAR fills Settings.application.testvar.
    // TODO: Could replace APP with the APP_ENVIRONMENT variable extracted above to pass
    // passwords to the production settings from the local environment.
    let env_vars = config::Environment::default()
        .prefix("APP")
        .prefix_separator("__")
        .separator("_");
    let builder = builder.add_source(env_vars);

    // Initialise our configuration reader
    let conf = builder.build()?;

    conf.try_deserialize()
}

// The "environment" struct.

pub enum Environment {
    Local,
    Production,
}
impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`.",
                other
            )),
        }
    }
}
