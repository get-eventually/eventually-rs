use std::net::SocketAddr;

use envconfig::Envconfig;

#[derive(Envconfig)]
pub(crate) struct Config {
    #[envconfig(from = "DB_HOST", default = "localhost")]
    pub db_host: String,

    #[envconfig(from = "DB_PORT", default = "5432")]
    pub db_port: u16,

    #[envconfig(from = "DB_USERNAME", default = "postgres")]
    pub db_username: String,

    #[envconfig(from = "DB_PASSWORD", default = "password")]
    pub db_password: String,

    #[envconfig(from = "DB_DATABASE", default = "postgres")]
    pub db_database: String,

    #[envconfig(from = "HTTP_PORT", default = "8080")]
    pub http_port: u16,
}

impl Config {
    pub(crate) fn postgres_dsn(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode=disable",
            self.db_username, self.db_password, self.db_host, self.db_port, self.db_database
        )
    }

    pub(crate) fn addr(&self) -> SocketAddr {
        ([0, 0, 0, 0], self.http_port).into()
    }
}
