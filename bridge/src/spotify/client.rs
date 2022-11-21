use anyhow::Result;
use chrono::{DateTime, Utc};
use rspotify::{
    model::{AdditionalType, ArtistId, CurrentlyPlayingContext, PlayHistory, TimeLimits},
    prelude::*,
    scopes, AuthCodeSpotify, Config, Credentials, OAuth, Token,
};
use std::{env, fs, path::PathBuf};

use scrobblify_domain::{
    bridge::spotify::SpotifyApi,
    models::{CurrentPlayingTrack, HistoryPlayedTrack, Tag},
};

#[derive(thiserror::Error, Debug)]
pub enum SpotifyError {
    #[error("failed to initialize auth")]
    Auth(#[from] rspotify::ClientError),
    #[error("failed to parse track response")]
    TrackResponse,
}

#[derive(Clone, Debug)]
struct SpotifyClientConfig {
    client_id: String,
    client_secret: String,
    auth_callback_uri: String,
}

impl SpotifyClientConfig {
    pub fn new_from_env() -> Self {
        let client_id = match env::var_os("SCROBBLIFY_SPOTIFY_CLIENT_ID") {
            Some(v) => v.into_string().unwrap(),
            None => panic!("SCROBBLIFY_SPOTIFY_CLIENT_ID is not set"),
        };

        let client_secret = match env::var_os("SCROBBLIFY_SPOTIFY_CLIENT_SECRET") {
            Some(v) => v.into_string().unwrap(),
            None => panic!("SCROBBLIFY_SPOTIFY_CLIENT_SECRET is not set"),
        };

        let auth_callback_uri = match env::var_os("SCROBBLIFY_SPOTIFY_AUTH_CALLBACK_URI") {
            Some(v) => v.into_string().unwrap(),
            None => panic!("SCROBBLIFY_SPOTIFY_AUTH_CALLBACK_URI is not set"),
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
    pub async fn new_from_env() -> Result<SpotifyClient> {
        let client_config = SpotifyClientConfig::new_from_env();
        Self::new(client_config).await
    }

    async fn new(client_config: SpotifyClientConfig) -> Result<SpotifyClient> {
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

        let client = SpotifyClient(AuthCodeSpotify::with_config(creds, oauth, config));

        if get_cache_path().exists() {
            client.with_token().await
        } else {
            Ok(client)
        }
    }

    async fn with_token(&self) -> Result<Self> {
        let token = load_token_from_cache()?;
        *self.0.token.lock().await.unwrap() = Some(token.clone());

        Ok(self.clone())
    }
}

#[async_trait::async_trait]
impl SpotifyApi for SpotifyClient {
    // Auth
    fn has_auth(&self) -> bool {
        get_cache_path().exists()
    }

    async fn get_auth_url(&self) -> Result<String> {
        let auth_url = self.0.get_authorize_url(true)?;
        Ok(auth_url)
    }

    async fn get_auth_token(&mut self, code: &str) -> Result<()> {
        self.0.request_token(code).await?;
        Ok(())
    }

    // API
    async fn get_currently_playing(&self) -> Result<Option<CurrentPlayingTrack>> {
        let cpt: Option<CurrentPlayingTrack> = match self
            .0
            .current_playing(None, Some(&[AdditionalType::Track]))
            .await?
        {
            Some(cp) => Some(
                <CurrentlyPlayingContext as TryInto<super::shims::CurrentPlayingTrack>>::try_into(
                    cp,
                )?
                .into(),
            ),
            None => None,
        };

        Ok(cpt)
    }

    async fn get_recently_played(
        &self,
        timestamp: DateTime<Utc>,
    ) -> Result<Vec<HistoryPlayedTrack>> {
        let time_limit = TimeLimits::After(timestamp);

        let items = self
            .0
            .current_user_recently_played(Some(50), Some(time_limit))
            .await?
            .items;

        let history: Vec<HistoryPlayedTrack> = items
            .into_iter()
            .map(|ph| <PlayHistory as TryInto<super::shims::HistoryPlayedTrack>>::try_into(ph))
            .filter(|ph| ph.is_ok())
            .map(|ph| ph.unwrap())
            .map(|ph| ph.into())
            .collect();
        Ok(history)
    }

    async fn get_tags(&self, artists_ids: Vec<&str>) -> Result<Vec<Tag>> {
        let artists_ids: Vec<ArtistId> = artists_ids
            .into_iter()
            .map(|artist_id| ArtistId::from_id(artist_id).unwrap())
            .collect();
        let artists = self.0.artists(&artists_ids).await?;

        let mut tags: Vec<Tag> = artists
            .into_iter()
            .flat_map(|fa| {
                fa.genres
                    .into_iter()
                    .map(|id| Tag { id })
                    .collect::<Vec<Tag>>()
            })
            .collect();

        tags.sort();
        tags.dedup();

        Ok(tags)
    }
}

fn load_token_from_cache() -> Result<Token> {
    let cache_path = get_or_create_cache_path();
    Ok(Token::from_cache(cache_path)?)
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
