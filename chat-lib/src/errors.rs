use thiserror::Error;
use yansi::{Color, Paint};

#[derive(Error, Debug)]
pub enum ChatErrs<'a> {
    #[error("File not sent...")]
    FileNotSentErr,

    #[error("Deserialization error: {0}")]
    DeserializationErr(String),

    #[error("{0} {1}")]
    StreamErr(String, &'a String),
}

impl<'a> ChatErrs<'a> {
    pub fn red(&self) -> String {
        Paint::new(self.to_string()).fg(Color::Red).to_string()
    }
}
