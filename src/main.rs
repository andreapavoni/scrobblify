use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use scrobblify::{
    bridge::spotify::SpotifyClient,
    core::{start_auto_scrobbling, App},
    db::repository::Repository,
    web::HttpUi,
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

    let app = Arc::new(Mutex::new(App::new(Box::new(db), spotify)));
    let http_ui = HttpUi::new(app.clone());

    start_auto_scrobbling(app.clone()).await;

    http_ui.serve_from_env().await;

    Ok(())
}
