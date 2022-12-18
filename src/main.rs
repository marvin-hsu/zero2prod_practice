use std::net::TcpListener;

use sqlx::{Connection, PgConnection, PgPool};
use zero2prod_practice::configuration::get_configuration;
use zero2prod_practice::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connection to Postgres.");

    let address = format!("0.0.0.0:{}", configuration.application_port);

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
