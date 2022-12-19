use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod_practice::configuration::get_configuration;
use zero2prod_practice::startup::run;
use zero2prod_practice::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool =
        PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
            .expect("Failed to connection to Postgres.");

    let address = format!("0.0.0.0:{}", configuration.application_port);

    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
