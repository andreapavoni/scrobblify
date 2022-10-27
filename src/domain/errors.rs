// #[derive(thiserror::Error, Debug)]
// #[error("Something went wrong.")]
// pub struct SpotifyError {
//     #[from]
//     source: anyhow::Error,
// }

pub type SpotifyResult<T> = Result<T, SpotifyError>;

#[derive(thiserror::Error, Debug)]
pub enum SpotifyError {
    #[error("Failed to initialize auth")]
    Auth(#[from] rspotify::ClientError),
    #[error("Failed to process auth callback")]
    Callback, // (#[from] PasswordError),
    #[error("Failed to get auth token")]
    GetToken,
    #[error("Failed to refresh auth token")]
    RefreshToken, //(#[from] DatabaseError),
    #[error("Failed to parse track response")]
    TrackResponse,
}
