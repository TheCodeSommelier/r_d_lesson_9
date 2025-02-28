use std::{net::TcpStream, thread};

use anyhow::{Context, Result};
mod chat;
mod server;
use chat::MessageType;
use server::{listen_and_accept, validate_host, validate_port};

fn main() -> Result<()> {
    let mut port = String::new();
    let mut host = String::new();

    println!("Please specify port (default = 11111):");
    std::io::stdin().read_line(&mut port)?;
    port = port.trim().to_string(); // Remove newline

    println!("Please specify host (default = 127.0.0.1):");
    std::io::stdin().read_line(&mut host)?;
    host = host.trim().to_string(); // Remove newline

    let port = validate_port(&port)?;
    let host = validate_host(&host)?;

    let address = format!("{}:{}", host, port);

    println!("Run as server or client?");
    let mut mode = String::new();
    std::io::stdin().read_line(&mut mode)?;

    let chat_thread = thread::Builder::new().name(String::from("chat-thread"));

    if mode.trim().to_lowercase() == "server" {
        println!("Init server...");
        listen_and_accept(&address).context("Failed to run server")?;
    } else {
        println!("Client mode: Type messages to send");
        let client_thread = chat_thread.spawn(move || {
            let mut stream = match TcpStream::connect(&address) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to connect: {}", e);
                    return;
                }
            };

            loop {
                let mut buf = String::new();
                match std::io::stdin().read_line(&mut buf) {
                    Ok(_) => (),
                    Err(e) => eprintln!("Invalid input: {e}"),
                }

                match MessageType::determine_outgoing_message(&buf.trim().to_string()) {
                    Ok(new_message) => {
                        if let Err(e) = new_message
                            .send_message(&mut stream)
                            .context("Failed to send message to the server")
                        {
                            eprintln!("Send error: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Something went wrong: {e}");
                    }
                }
            }
        })?;
        client_thread.join().unwrap();
    }
    Ok(())
}
