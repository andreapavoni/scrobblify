use anyhow::Result;

use scrobblify_bridge::spotify::SpotifyClient;
use scrobblify_core::{App, Scrobbler};
use scrobblify_db::Repository;
use scrobblify_web::HttpUi;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "scrobblify=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Repository::new_from_env()
        .await
        .expect("failed to create repository");

    let spotify = SpotifyClient::new_from_env()
        .await
        .expect("failed to initialize spotify client");

    let app = Arc::new(Mutex::new(App::new(Box::new(db), spotify)));
    let http_ui = HttpUi::new(app.clone());

    Scrobbler::scrobble_recently_played(app.clone()).await;
    Scrobbler::start_auto_scrobbling(app.clone()).await;
    http_ui.serve_from_env().await;

    Ok(())
}
