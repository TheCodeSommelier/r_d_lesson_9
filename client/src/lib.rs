use anyhow::{Context, Result};
use yansi::Paint;

use std::io::{stdout, Write};
use std::net::TcpStream;
use std::thread;

use chat_lib::MessageType;

pub fn client(address: &str) -> Result<()> {
    let stream = TcpStream::connect(&address).context("Failed to connect".red())?;

    let receiver_stream = stream.try_clone().context("Failed to clone stream".red())?;
    let receiver_thread = thread::Builder::new()
        .name(String::from("receiver-thread"))
        .spawn(move || {
            println!("{}", "Listening for incoming messages...".green());
            loop {
                match MessageType::receive_message(&receiver_stream) {
                    Ok(message) => {
                        match message {
                            MessageType::Text(text) => {
                                println!("\n{} {}", "Received:".green(), text);
                            }
                            MessageType::Image(_) => {
                                println!("\n{}", "Received an image".green());
                            }
                            MessageType::File { name, .. } => {
                                println!("\n{} {}", "Received file:".green(), name);
                            }
                        }

                        print!("{}", "> ".yellow());
                        stdout().flush().expect("could not flush stdout");
                    }
                    Err(e) => {
                        eprintln!("\nError receiving message: {}", e.red());
                        if e.to_string().contains("end of file") {
                            eprintln!("{}", "Server connection closed. Exiting...".red());
                            break;
                        }
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
                    eprintln!("Invalid input: {}", e.red());
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
                        .context("Failed to send message to the server".red())
                    {
                        eprintln!("Send error: {}", e.red());
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Something went wrong: {}", e.red());
                }
            }
        })?;
    sender_thread.join().unwrap();
    receiver_thread.join().unwrap();

    Ok(())
}
