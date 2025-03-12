use std::net::TcpListener;

use crate::routes::health_check;
use crate::routes::subscribe;
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;

pub fn run(listener: TcpListener, connection: PgPool) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let server = HttpServer::new(move || {
        App::new()
            //    .wrap(Logger::default())
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            // 注册一个连接作为程序状态的一部分
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
