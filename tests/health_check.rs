use std::net::TcpListener;

#[actix_rt::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}

fn spawn_app() -> String {
    let listner = TcpListener::bind("127.0.0.1:0").expect("Failed to bind address");
    let port = listner.local_addr().unwrap().port();
    let server = zero2prod::run(listner).expect("Failed to get server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
