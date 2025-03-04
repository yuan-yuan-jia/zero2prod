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
    // Spans, like logs,have an associated level
    // `info_span` creates a span at the info_level
    // 我们不再使用字符串插值：跟踪允许我们将结构化信息作为键值对集合关联到我们的跨度中。
    // 我们可以明确命名它们（例如subscriber_email用于form.email），或者隐式使用变量名作为键（例如独立的request_id等同于request_id = request_id）。
    // 我们在它们前面都加了一个%符号：这是告诉tracing使用它们的Display实现来进行日志记录
    let request_span = tracing::info_span!("Adding a new subscriber.",
            %request_id,
            subscriber_email =  %form.email,
            subscriber_name  =  %form.name
    );
    //using `enter` in an async function is not a good idea
    //using `enter` there is just 
    // `info_span` 返回新创建的 span，但我们必须显式使用 `.enter()` 方法进入它以激活它。
    // `.enter()` 返回一个 `Entered` 实例，这是一个守卫：只要守卫变量未被释放，所有下游的跨度和日志事件都将被注册为已进入跨度的子项。
    let _request_span_guard = request_span.enter();
    tracing::info!("request_id {} - Saving new subscriber details in the database", request_id);
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
        tracing::info!("request_id {} - New subscriber details have been saved", request_id);
            HttpResponse::Ok().finish() 
        },
        Err(e) => {
            tracing::error!("request_id {} - Failed to execute query: {}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
