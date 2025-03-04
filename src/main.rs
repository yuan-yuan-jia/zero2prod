use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter,Registry};
use tracing_log::LogTracer;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // 重定向所有的`log`'s enves 到 subscriber
    LogTracer::init().expect("Failed to set logger");
    //env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
        
    let env_filter = EnvFilter::try_from_default_env()
                                .unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);
    let subscriber = Registry::default()
            .with(env_filter)
            .with(JsonStorageLayer)
            .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);

    let listner = TcpListener::bind(address)?;
    let connections = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    run(listner, connections)
        .expect("Failed to get server")
        .await
}
