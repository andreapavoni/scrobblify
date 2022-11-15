// #[derive(thiserror::Error, Debug)]
// #[error("Something went wrong.")]
// pub struct SpotifyError {
//     #[from]
//     source: anyhow::Error,
// }

#[derive(thiserror::Error, Debug)]
#[error("database error")]
pub struct DatabaseError {
    #[from]
    source: anyhow::Error,
}

// pub type SpotifyResult<T> = Result<T, SpotifyError>;

#[derive(thiserror::Error, Debug)]
pub enum SpotifyError {
    #[error("failed to initialize auth")]
    Auth(#[from] rspotify::ClientError),
    #[error("failed to process auth callback")]
    Callback, // (#[from] PasswordError),
    #[error("failed to get auth token")]
    GetToken,
    #[error("failed to refresh auth token")]
    RefreshToken, //(#[from] DatabaseError),
    #[error("failed to parse track response")]
    TrackResponse,
}
