use crate::domain::NewSubscriber;
use crate::domain::SubscriberEmail;
use crate::domain::SubscriberName;
use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;
// 一个扩展的trait，提供了`graphemes` 方法
// 为`String`和`&str`

// Instrument::instrument 完全按照我们的期望执行：
// 每次轮询 self（即 future）时，
// 它会进入我们传递的参数中的 span；
// 每次 future 暂停时，它会退出该 span。

#[derive(serde::Deserialize)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id,email,name,subscribed_at)
        VALUES ($1,$2,$3,$4)
    "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.inner_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

// 使用类属性宏来启动一个span
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form,connection),
    fields(
//        request_id = %Uuid::new_v4(),
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    )
)]
pub async fn subscribe(form: web::Form<FormData>, connection: web::Data<PgPool>) -> impl Responder {
    //  let request_id = Uuid::new_v4();
    // Spans, like logs,have an associated level
    // `info_span` creates a span at the info_level
    // 我们不再使用字符串插值：跟踪允许我们将结构化信息作为键值对集合关联到我们的跨度中。
    // 我们可以明确命名它们（例如subscriber_email用于form.email），或者隐式使用变量名作为键（例如独立的request_id等同于request_id = request_id）。
    // 我们在它们前面都加了一个%符号：这是告诉tracing使用它们的Display实现来进行日志记录
    //let request_span = tracing::info_span!("Adding a new subscriber.",
    //        %request_id,
    //        subscriber_email =  %form.email,
    //        subscriber_name  =  %form.name
    //);
    //using `enter` in an async function is not a good idea
    //using `enter` there is just
    // `info_span` 返回新创建的 span，但我们必须显式使用 `.enter()` 方法进入它以激活它。
    // `.enter()` 返回一个 `Entered` 实例，这是一个守卫：只要守卫变量未被释放，所有下游的跨度和日志事件都将被注册为已进入跨度的子项。
    //let _request_span_guard = request_span.enter();
    //tracing::info!("request_id {} - Saving new subscriber details in the database", request_id);

    // 我们不在 `query_span` 上调用 `.enter`！`.instrument` 会在查询未来的生命周期中的适当时刻处理它。
    //   let query_span = tracing::info_span!("Saving new subsriber details in the database");
    //   let result = sqlx::query!(
    //       r#"
    //       INSERT INTO subscriptions (id,email,name,subscribed_at)
    //       VALUES ($1,$2,$3,$4)
    //   "#,
    //       Uuid::new_v4(),
    //       form.email,
    //       form.name,
    //       Utc::now()
    //   )
    //   .execute(connection.get_ref())
    //   // First we attach the instrumentation,then we `.await` it
    //   .instrument(query_span)
    //   .await;
    let subscriber = match form.0.try_into() {
        Ok(r) => r,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    match insert_subscriber(&connection, &subscriber).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => {
            // this error log falls outside of `query_span`
            // tracing::error!("request_id {} - Failed to execute query: {}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
    let name = SubscriberName::parse(form.name)?;
    let email = SubscriberEmail::parse(form.email)?;

    Ok(NewSubscriber { email, name })
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}
