#[derive(thiserror::Error, Debug)]
#[error("database error")]
pub struct DatabaseError {
    #[from]
    source: anyhow::Error,
}
