use config::{Config, File, FileFormat};

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    // Method to join together fields into a single connection string.
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }
}

// Read configuration settings from configuration.yaml and return them as a Settings object.
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    // Initialise our configuration reader
    let builder = Config::builder().add_source(File::new("configuration", FileFormat::Yaml));
    let conf = builder.build()?;
    conf.try_deserialize()
}
