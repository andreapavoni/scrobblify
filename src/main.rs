use anyhow::Result;
use std::{env, sync::Arc};
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use scrobblify::{
    core::{spotify::SpotifyClient, start_scrobbling, App},
    db::repository::Repository,
    web::http::new_app,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Repository::new_from_env()
        .await
        .expect("Failed to create repository");

    let spotify = SpotifyClient::new_from_env()
        .await
        .expect("Failed to initialize spotify client");

    let core = Arc::new(Mutex::new(App::new(Box::new(db), spotify)));

    tokio::spawn(async move {
        let _ = start_scrobbling(core.clone()).await;
    });

    let app = new_app().await;
    let host = env::var("SCRUBBLIFY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("{}:{}", host, port);
    app.run(addr.as_str()).await;

    Ok(())
}
