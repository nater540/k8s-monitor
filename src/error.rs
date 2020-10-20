#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("JSON serialization failed")]
  SerializationFailed(#[from] serde_json::Error),

  #[error("ruh-roh... reconciliation failed")]
  ReconcileFailed
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
