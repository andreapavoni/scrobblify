use askama::Template;
use axum::{
    extract::{Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::{env, net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::domain::app::App as DomainApp;

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
            .layer(SetResponseHeaderLayer::if_not_present(
                header::SERVER,
                HeaderValue::from_static("scrobblify"),
            ));

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

    HtmlTemplate(HomeTemplate {}).into_response()
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
struct HomeTemplate {}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    error: String,
}
