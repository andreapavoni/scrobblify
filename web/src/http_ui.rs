use askama::Template;
use axum::{
    extract::{Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use axum_extra::routing::SpaRouter;
use chrono::NaiveDate;
use serde::Deserialize;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{
    set_header::SetResponseHeaderLayer,
    trace::{DefaultOnResponse, TraceLayer},
    LatencyUnit,
};

use scrobblify_domain::{
    app::App as DomainApp,
    db::ParamsForStatsQuery,
    models::{StatsArtist, StatsTag, StatsTrack},
};

type App = Arc<Mutex<dyn DomainApp>>;

// HTTP interaface to the app
pub struct HttpUi {
    router: Router<App>,
}

impl HttpUi {
    pub fn new(app: Arc<Mutex<dyn DomainApp>>) -> Self {
        let router = Router::with_state(app.clone())
            .route("/auth/callback", get(auth_callback_handler))
            .route("/", get(index_handler))
            .merge(SpaRouter::new("/assets", "web/assets"))
            .layer(SetResponseHeaderLayer::if_not_present(
                header::SERVER,
                HeaderValue::from_static("scrobblify"),
            ))
            .layer(SetResponseHeaderLayer::if_not_present(
                header::CACHE_CONTROL,
                HeaderValue::from_static("max-age=3600"),
            ))
            .layer(
                TraceLayer::new_for_http()
                    .on_response(DefaultOnResponse::new().latency_unit(LatencyUnit::Micros)),
            );

        HttpUi { router }
    }

    pub async fn serve_from_env(&self) {
        let host = env::var("SCRUBBLIFY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());

        self.serve(host, port).await;
    }

    pub async fn serve(&self, host: String, port: String) {
        let addr = format!("{}:{}", host, port)
            .parse::<SocketAddr>()
            .expect(format!("unable to parse socket address with `{}:{}`", host, port).as_str());

        tracing::info!(msg = "server started", address = addr.to_string());

        axum::Server::bind(&addr)
            .serve(self.router.clone().into_make_service())
            .await
            .unwrap();
    }
}

// Handlers
async fn index_handler(State(app): State<App>) -> Response {
    if !app.lock().await.is_spotify_authenticated() {
        let auth_url = app.lock().await.get_spotify_auth_url().await.unwrap();

        // OAuth2 step 1: send user to Spotify auth page
        return HtmlTemplate(AuthorizeTemplate { auth_url }).into_response();
    }

    let opts = ParamsForStatsQuery {
        start: NaiveDate::from_ymd(2022, 11, 1),
        end: NaiveDate::from_ymd(2022, 11, 15),
        limit: None,
    };

    let top_tracks = app
        .lock()
        .await
        .stats_for_popular_tracks(opts.clone())
        .await;
    let top_artists = app
        .lock()
        .await
        .stats_for_popular_artists(opts.clone())
        .await;
    let top_tags = app.lock().await.stats_for_popular_tags(opts.clone()).await;

    HtmlTemplate(HomeTemplate {
        top_tracks,
        top_artists,
        top_tags,
    })
    .into_response()
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AuthCallbackParams {
    code: Option<String>,
}

async fn auth_callback_handler(
    Query(params): Query<AuthCallbackParams>,
    State(app): State<App>,
) -> impl IntoResponse {
    // OAuth2 step 2: user is redirected to callback with a `code`
    let code = params.code.unwrap();
    // OAuth2 step 3: fetch the token/refresh for API requests
    let _ = app.lock().await.store_spotify_auth_token(&code).await;

    Redirect::to("/").into_response()
}

// Templates
struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                HtmlTemplate(ErrorTemplate {
                    error: error.to_string(),
                }),
            )
                .into_response(),
        }
    }
}

#[derive(Template)]
#[template(path = "authorize.html")]
struct AuthorizeTemplate {
    auth_url: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    pub top_tracks: Vec<StatsTrack>,
    pub top_tags: Vec<StatsTag>,
    pub top_artists: Vec<StatsArtist>,
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    error: String,
}

pub mod filters {
    use std::time::Duration;

    use crate::utils::secs_to_hours_and_minutes;

    pub fn fmt_secs_to_hhmm(d: &f64) -> askama::Result<String> {
        let duration_secs = Duration::from_secs_f64(*d);
        let (hours, minutes) = secs_to_hours_and_minutes(duration_secs);

        Ok(format!("{:02}:{:02}", hours, minutes))
    }
}
