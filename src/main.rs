use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    //env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listner = TcpListener::bind(address)?;
    let connections = PgPool::connect(&configuration.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");
    run(listner, connections)
        .expect("Failed to get server")
        .await
}
