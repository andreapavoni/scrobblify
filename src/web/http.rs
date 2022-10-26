use super::spotify::SpotifyClient;
use askama::Template;
use graphul::{
    http::{response::Redirect, Methods},
    template::HtmlTemplate,
    Context, Graphul, IntoResponse,
};

#[derive(Template)]
#[template(path = "authorize.html")]
struct AuthorizeTemplate {
    auth_url: String,
}

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    currently_playing: String,
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
        let mut spotify = SpotifyClient::new();

        // OAuth2 step 3: fetch the token/refresh for API requests
        spotify.get_auth_token(code.as_str()).await;

        return Redirect::to("/").into_response();
    });

    app.get("/", |_c: Context| async move {
        if !SpotifyClient::has_auth() {
            let spotify = SpotifyClient::new();
            let auth_url = spotify.get_auth_url().await;

            // OAuth2 step 1: send user to Spotify auth page
            let template = AuthorizeTemplate { auth_url };
            return HtmlTemplate(template).into_response();
        }

        let spotify = SpotifyClient::from_cache().await;

        let recently_played = spotify.get_recently_played().await;
        println!("RECENTLY PLAYED: {:#?}", recently_played);

        return match spotify.get_currently_playing().await {
            Ok(resp) => {
                let currently_playing = format!("{:#?}", resp);

                let template = HomeTemplate { currently_playing };
                HtmlTemplate(template).into_response()
            }

            Err(err) => {
                let template = ErrorTemplate {
                    error: format!("Failed for {err}!"),
                };
                HtmlTemplate(template).into_response()
            }
        };
    });

    app
}
