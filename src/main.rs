use std::net::TcpListener;
use zero2prod::run;
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let listner = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind address");
    run(listner).expect("Failed to get server").await
}
