use anyhow::Result;
use std::{env, sync::Arc};
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use scrobblify::{
    bridge::spotify::SpotifyClient,
    core::{auto_scrobble, App},
    db::repository::Repository,
    web::http::new_app,
};

const SPOTIFY_POLLING_SECS: u64 = 60;

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
        loop {
            if let Err(err) = auto_scrobble(core.clone()).await {
                tracing::error!("error while scrobbling: `{:?}`", err)
            }

            println!("======= sleep ========");
            let duration = Duration::new(SPOTIFY_POLLING_SECS, 0);
            sleep(duration).await;
        }
    });

    let app = new_app().await;
    let host = env::var("SCRUBBLIFY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("{}:{}", host, port);
    app.run(addr.as_str()).await;

    Ok(())
}
