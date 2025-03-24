use anyhow::Error;
use thiserror::Error;
use yansi::{Color, Paint};

#[derive(Error, Debug)]
pub enum ClientErrs {
    #[error("Server connection closed: {0}")]
    ServerClosedErr(#[source] Error),

    #[error("Invalid input: {0}")]
    InvalidInputErr(#[source] Error),

    #[error("Send error: {0}")]
    SendErr(#[source] Error),

    #[error("Something went wrong: {0}")]
    GenericErr(#[source] Error),
}

impl ClientErrs {
    pub fn red(&self) -> String {
        Paint::new(self.to_string()).fg(Color::Red).to_string()
    }
}
