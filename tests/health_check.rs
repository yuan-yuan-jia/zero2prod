use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Executor, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings, Settings},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[actix_rt::test]
async fn health_check_works() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request");
    drop_database(test_app).await;
    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=urlula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email,name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");
    drop_database(test_app).await;
    assert_eq!(saved.email, "urlula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the emial"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}",
            error_message
        );
    }
    drop_database(test_app).await;
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub settings: Settings,
}

async fn drop_database(test_app: TestApp) {
    if let Ok(p) = PgPool::connect(
        &test_app
            .settings
            .database
            .connection_string_without_db()
            .expose_secret(),
    )
    .await
    {
        let sql = format!(
            r#"drop database "{}";"#,
            &test_app.settings.database.database_name
        );
        let _ = p.execute(sql.as_str()).await;
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut pool = PgPool::connect(&config.connection_string_without_db().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    pool.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name.as_str()).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect Postgres");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate the database");

    pool
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listner = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listner.local_addr().unwrap().port();
    let address = format!("http:127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection = configure_database(&configuration.database).await;
    let server =
        zero2prod::startup::run(listner, connection.clone()).expect("Failed to get server");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection,
        settings: configuration,
    }
}
