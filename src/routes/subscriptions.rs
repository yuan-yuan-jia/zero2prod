use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();
    log::info!("request_id {} - Adding '{}' '{}' as a new subscriber.",
            request_id,
            form.email,
            form.name
    );
    log::info!("request_id {} - Saving new subscriber details in the database", request_id);
    let result = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id,email,name,subscribed_at)
        VALUES ($1,$2,$3,$4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .await;

    match result {
        Ok(_) => {
        log::info!("request_id {} - New subscriber details have been saved", request_id);
            HttpResponse::Ok().finish() 
        },
        Err(e) => {
            log::error!("request_id {} - Failed to execute query: {}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
