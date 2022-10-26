use chrono::Utc;
use rspotify::{
    model::{
        AdditionalType, CurrentlyPlayingContext, CursorBasedPage, FullTrack, PlayHistory,
        TimeLimits, TrackId,
    },
    prelude::*,
    scopes, AuthCodeSpotify, ClientError, Config, Credentials, OAuth, Token,
};
use std::{env, fs, path::PathBuf};
use tracing::log;

#[derive(Clone, Debug)]
struct SpotifyClientConfig {
    client_id: String,
    client_secret: String,
    auth_callback_uri: String,
}

impl SpotifyClientConfig {
    pub fn new() -> Self {
        let client_id = match env::var_os("SCROBBLIFY_SPOTIFY_CLIENT_ID") {
            Some(v) => v.into_string().unwrap(),
            None => panic!("$SCROBBLIFY_SPOTIFY_CLIENT_ID is not set"),
        };

        let client_secret = match env::var_os("SCROBBLIFY_SPOTIFY_CLIENT_SECRET") {
            Some(v) => v.into_string().unwrap(),
            None => panic!("$SCROBBLIFY_SPOTIFY_CLIENT_SECRET is not set"),
        };

        let auth_callback_uri = match env::var_os("SCROBBLIFY_SPOTIFY_AUTH_CALLBACK_URI") {
            Some(v) => v.into_string().unwrap(),
            None => panic!("$SCROBBLIFY_SPOTIFY_AUTH_CALLBACK_URI is not set"),
        };

        Self {
            client_id,
            client_secret,
            auth_callback_uri,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpotifyClient(AuthCodeSpotify);

impl SpotifyClient {
    pub fn new() -> SpotifyClient {
        let client_config = SpotifyClientConfig::new();
        let creds = Credentials::new(
            client_config.client_id.as_str(),
            client_config.client_secret.as_str(),
        );

        let oauth = OAuth {
            scopes: scopes!(
                "user-read-recently-played",
                "user-read-playback-state",
                "user-read-currently-playing"
            ),
            // URL must be the same of the one configured in Spotify app
            redirect_uri: client_config.auth_callback_uri,
            ..Default::default()
        };

        let config = Config {
            token_cached: true,
            cache_path: get_or_create_cache_path(),
            token_refreshing: true,
            ..Default::default()
        };

        SpotifyClient(AuthCodeSpotify::with_config(creds, oauth, config))
    }

    pub async fn from_cache() -> SpotifyClient {
        let spotify = SpotifyClient::new().with_token().await;

        spotify
            .0
            .refresh_token()
            .await
            .expect("couldn't refresh user token");

        spotify
    }

    pub fn has_auth() -> bool {
        get_cache_path().exists()
    }

    async fn with_token(&self) -> Self {
        let token = load_token_from_cache();
        *self.0.token.lock().await.unwrap() = Some(token.clone());

        self.clone()
    }

    // Auth
    pub async fn get_auth_url(&self) -> String {
        self.0.get_authorize_url(true).unwrap()
    }

    pub async fn get_auth_token(&mut self, code: &str) {
        match self.0.request_token(code).await {
            Ok(_) => {
                // this is the token, we might store it on db rather than on json file?
                // self.0.load_token().lock().await.unwrap().clone().unwrap()
            }
            Err(err) => {
                log::error!("Failed to get user token {:?}", err);
            }
        }
    }

    // API
    pub async fn get_currently_playing(
        &self,
    ) -> Result<Option<CurrentlyPlayingContext>, ClientError> {
        self.0
            .current_playing(None, Some(&[AdditionalType::Track]))
            .await
    }

    pub async fn get_recently_played(&self) -> Result<CursorBasedPage<PlayHistory>, ClientError> {
        let time_limit = TimeLimits::Before(Utc::now());

        self.0
            .current_user_recently_played(Some(20), Some(time_limit))
            .await
    }

    pub async fn get_track(&self, track_id: &str) -> Result<FullTrack, ClientError> {
        self.0.track(&TrackId::from_id(track_id).unwrap()).await
    }
}

fn load_token_from_cache() -> Token {
    let cache_path = get_or_create_cache_path();
    Token::from_cache(cache_path).unwrap()
}

fn get_cache_path() -> PathBuf {
    let project_dir_path = env::current_dir().unwrap();
    let mut cache_path = project_dir_path;
    cache_path.push(".spotify_cache/");
    cache_path.push("scrobblify");

    cache_path
}

fn get_or_create_cache_path() -> PathBuf {
    let cache_path = get_cache_path();
    if !cache_path.exists() {
        let mut path = cache_path.clone();
        path.pop();
        fs::create_dir_all(path).unwrap();
    }
    cache_path
}
