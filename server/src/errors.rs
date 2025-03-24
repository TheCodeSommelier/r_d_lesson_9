use anyhow::Error;
use std::net::SocketAddr;
use thiserror::Error;
use yansi::{Color, Paint};

#[derive(Error, Debug)]
pub enum ServerErrs {
    #[error("Client cannot connect: {0}")]
    ClientConnectionErr(#[source] Error),

    #[error("Error receiving message: {0}")]
    MessageReceivingErr(#[source] Error),

    #[error("Error writing message to client {0}: {1}")]
    MessageWritingErr(SocketAddr, #[source] Error),
}

impl ServerErrs {
    pub fn red(&self) -> String {
        Paint::new(self.to_string()).fg(Color::Red).to_string()
    }
}
