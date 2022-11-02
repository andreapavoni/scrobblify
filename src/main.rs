use std::{env, time::Duration};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use scrobblify::db::repository::Repository;
use scrobblify::domain::repository::Repository as DomainRepository;
use scrobblify::web::http::new_app;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = env::var("DATABASE_URL").unwrap();
    let db = Repository::new(db_url)
        .await
        .expect("Failed to create repository");

    let track = scrobblify::domain::models::Track {
        title: String::from("test"),
        id: String::from("1234566"),
        duration_secs: Duration::from_secs_f64(220.30),
    };
    match db.insert_track(track.clone()).await {
        Ok(_) => println!("==== insert ok ===="),
        Err(e) => println!("==== insert error: `{:#?}` ====", e),
    }

    match db.get_track_by_id(track.id).await {
        Ok(m) => println!("==== find ok: {:?} ====", m),
        Err(e) => println!("==== find error: `{:#?}` ====", e),
    }

    match db.get_track_by_id(String::from("abcde")).await {
        Ok(m) => println!("==== find 2 ok: {:?} ====", m),
        Err(e) => println!("==== find 2 error: `{:#?}` ====", e),
    }

    let app = new_app().await;
    let host = env::var("SCRUBBLIFY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("{}:{}", host, port);
    app.run(addr.as_str()).await;
}
