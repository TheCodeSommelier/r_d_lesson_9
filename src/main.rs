use anyhow::{Context, Result};
use std::io::stdin;
use yansi::Paint;

use client::client;
use server::{listen_and_accept, validate_host, validate_port};

fn main() -> Result<()> {
    let mut port = String::new();
    let mut host = String::new();

    println!(
        "Please specify {} (default = 11111):",
        "port".bold().green()
    );
    std::io::stdin().read_line(&mut port)?;
    port = port.trim().to_string();

    println!(
        "Please specify {} (default = 127.0.0.1):",
        "host".bold().green()
    );
    std::io::stdin().read_line(&mut host)?;
    host = host.trim().to_string();

    let port = validate_port(&port)?;
    let host = validate_host(&host)?;
    let address = format!("{}:{}", host, port);

    println!("Run as server or client?");
    let mut mode = String::new();
    stdin().read_line(&mut mode)?;

    if mode.trim().to_lowercase() == "server" {
        listen_and_accept(&address).context("Failed to run server".red().bold())?;
    } else {
        println!("Client mode: {}", "Type messages to send".green());
        client(&address).context("Failed to run client".red().bold())?;
    }

    Ok(())
}
