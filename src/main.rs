use anyhow::{Context, Result};
use std::{net::TcpStream, thread};
mod chat;
mod server;
use chat::MessageType;
use server::{listen_and_accept, validate_host, validate_port};

fn main() -> Result<()> {
    let mut port = String::new();
    let mut host = String::new();

    println!("Please specify port (default = 11111):");
    std::io::stdin().read_line(&mut port)?;
    port = port.trim().to_string();

    println!("Please specify host (default = 127.0.0.1):");
    std::io::stdin().read_line(&mut host)?;
    host = host.trim().to_string();

    let port = validate_port(&port)?;
    let host = validate_host(&host)?;
    let address = format!("{}:{}", host, port);

    println!("Run as server or client?");
    let mut mode = String::new();
    std::io::stdin().read_line(&mut mode)?;

    if mode.trim().to_lowercase() == "server" {
        listen_and_accept(&address).context("Failed to run server")?;
        println!("Server initialized!");
    } else {
        println!("Client mode: Type messages to send");

        let stream = match TcpStream::connect(&address) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to connect: {}", e);
                return Err(e.into());
            }
        };

        let receiver_stream = stream.try_clone().context("Failed to clone stream")?;
        let receiver_thread = thread::Builder::new()
            .name(String::from("receiver-thread"))
            .spawn(move || {
                println!("Listening for incoming messages...");
                loop {
                    match MessageType::receive_message(&receiver_stream) {
                        Ok(message) => match message {
                            MessageType::Text(text) => {
                                println!("\nReceived: {}", text);
                                print!("> ");
                                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            }
                            MessageType::Image(_) => {
                                println!("\nReceived an image");
                                print!("> ");
                                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            }
                            MessageType::File { name, .. } => {
                                println!("\nReceived file: {}", name);
                                print!("> ");
                                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                            }
                            MessageType::Empty => {
                                eprintln!("This was not supposed to happen...")
                            }
                        },
                        Err(e) => {
                            eprintln!("\nError receiving message: {}", e);
                            if e.to_string().contains("end of file") {
                                eprintln!("Server connection closed. Exiting...");
                                break;
                            }
                            // Continue trying for other errors
                        }
                    }
                }
            })?;

        let mut sender_stream = stream;
        let sender_thread = thread::Builder::new()
            .name(String::from("sender-thread"))
            .spawn(move || loop {
                let mut buf = String::new();
                match std::io::stdin().read_line(&mut buf) {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Invalid input: {e}");
                        continue;
                    }
                }

                let trimmed = buf.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }

                match MessageType::determine_outgoing_message(&trimmed) {
                    Ok(new_message) => {
                        if let Err(e) = new_message
                            .send_message(&mut sender_stream)
                            .context("Failed to send message to the server")
                        {
                            eprintln!("Send error: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Something went wrong: {e}");
                    }
                }
            })?;
        sender_thread.join().unwrap();
        receiver_thread.join().unwrap();
    }

    Ok(())
}
