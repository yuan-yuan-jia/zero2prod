use std::net::TcpListener;

use sqlx::{Connection, PgConnection, PgPool};
use zero2prod::configuration::get_configuration;

#[actix_rt::test]
async fn health_check_works() {
    let address = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app().await;
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to get connection of pg");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=urlula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email,name FROM subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email , "urlula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the emial"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app_address))
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
}

async  fn spawn_app() -> String {
    let listner = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listner.local_addr().unwrap().port();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection = PgPool::connect(
        &configuration.database.connection_string()
    ).await
    .expect("Failed to connect to Postgres");
    let server = zero2prod::startup::run(listner, connection).expect("Failed to get server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
