use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};

async fn greet(req: HttpRequest) -> impl HttpResponder {
    let name = req.match_info().get("name").unwrap_or("world");
    format!("Hello {}!", name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(greet))
            .route("/{name}", web::get().to(greet))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
