use askama::Template;
use graphul::{
    http::{response::Redirect, Methods},
    template::HtmlTemplate,
    Context, Graphul, IntoResponse,
};

use crate::core::spotify::SpotifyClient;

#[derive(Template)]
#[template(path = "authorize.html")]
struct AuthorizeTemplate {
    auth_url: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    currently_playing: String,
    recently_played: String,
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    error: String,
}

pub async fn new_app() -> Graphul {
    let mut app = Graphul::new();

    // OAuth2 step 2: user is redirected to callback with a `code`
    app.get("/auth/callback/", |c: Context| async move {
        let code = c.query("code");
        let mut spotify = SpotifyClient::new_from_env().await.unwrap();

        // OAuth2 step 3: fetch the token/refresh for API requests
        let _ = spotify.get_auth_token(code.as_str()).await;

        Redirect::to("/").into_response()
    });

    app.get("/", |_c: Context| async move {
        let spotify = SpotifyClient::new_from_env().await.unwrap();

        if !spotify.has_auth() {
            let auth_url = spotify.get_auth_url().await.unwrap();

            // OAuth2 step 1: send user to Spotify auth page
            return HtmlTemplate(AuthorizeTemplate { auth_url }).into_response();
        }

        let recently_played = spotify.get_recently_played().await.unwrap();
        let currently_playing = spotify.get_currently_playing().await;

        let recently_played = format!("{:#?}", recently_played);
        let currently_playing = format!("{:#?}", currently_playing);

        HtmlTemplate(HomeTemplate {
            currently_playing,
            recently_played,
        })
        .into_response()
    });

    app
}
