use anyhow::Error;
use thiserror::Error;
use yansi::{Color, Paint};

#[derive(Error, Debug)]
pub enum ClientErrs<'a> {
    #[error("\nError receiving message: {0}")]
    ServerClosedErr(&'a String),

    #[error("Invalid input: {0}")]
    InvalidInputErr(#[source] Error),

    #[error("Send error: {0}")]
    SendErr(#[source] Error),

    #[error("Something went wrong: {0}")]
    GenericErr(#[source] Error),
}

impl<'a> ClientErrs<'a> {
    pub fn red(&self) -> String {
        Paint::new(self.to_string()).fg(Color::Red).to_string()
    }
}
