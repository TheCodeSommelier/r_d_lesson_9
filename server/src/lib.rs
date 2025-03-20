use anyhow::{anyhow, Result};
use regex::Regex;
use yansi::Paint;

use std::collections::HashMap;
use std::net::TcpListener;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use chat_lib::MessageType;

pub fn validate_port(port: &str) -> Result<String> {
    /*
    if port.is_empty() {
        return "11111".to_string();
    }

    port.parse::<u16>()
        .context("The port is invalid")?;

    Ok(port.to_string())
    */

    let regex = Regex::new(r"[0-9]")?;

    if regex.is_match(port) {
        Ok(port.to_string())
    } else if port.is_empty() {
        Ok("11111".to_string())
    } else {
        Err(anyhow!("The port is invalid...".red()))
    }
}

pub fn validate_host(host: &str) -> Result<String> {
    let regex = Regex::new(
        r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$",
    )?;

    if regex.is_match(host) {
        Ok(host.to_string())
    } else if host == "" {
        Ok("127.0.0.1".to_string())
    } else {
        Err(anyhow!("The host is invalid...".red()))
    }
}

pub fn listen_and_accept(address: &String) -> Result<()> {
    let listener = TcpListener::bind(address)?;
    let clients = Arc::new(Mutex::new(HashMap::new()));

    println!("\n{}", "Server initialized!".green());

    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue;
        };

        let Ok(addr) = stream.peer_addr() else {
            continue;
        };

        println!("New client connected: {}", addr.blue().bright().underline());

        let clients_clone = Arc::clone(&clients);

        {
            let mut clients_map = clients.lock().unwrap();
            clients_map.insert(addr, stream.try_clone()?);
        }

        thread::spawn(move || {
            if let Err(e) = handle_client(stream, addr, clients_clone) {
                println!("Client error: {}", e);
            }
        });
    }

    Ok(())
}

fn handle_client(
    stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
) -> Result<()> {
    loop {
        match MessageType::receive_message(&stream) {
            Ok(message) => {
                match &message {
                    MessageType::Text(text_content) => {
                        println!("{text_content}");
                    }
                    MessageType::Image(image_data) => {
                        println!("{}", "Receiving image...".green());

                        let file_name = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let path = format!("./images/{file_name}.png");
                        MessageType::save_file_to_disk(path, &image_data)?;
                    }
                    MessageType::File { name, content } => {
                        println!("Received file: {}", name);
                        let path = format!("./files/{name}");
                        MessageType::save_file_to_disk(path, &content)?;
                    }
                }
                broadcast(&clients, &addr, message)?;
            }
            Err(e) => {
                println!("Error receiving message: {}", e.red());
            }
        }
    }
}

fn broadcast(
    clients: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    sender_addr: &SocketAddr,
    msg: MessageType,
) -> Result<()> {
    let mut to_remove = Vec::new();
    let mut clients_map = clients.lock().unwrap();

    for (&addr, mut stream) in clients_map.iter_mut() {
        if addr == *sender_addr {
            continue;
        }

        match msg.send_message(&mut stream) {
            Ok(_) => {
                println!("Successfully sent message to client {}", addr.green());
            }
            Err(e) => {
                println!(
                    "Error writing message to client {}: {}",
                    addr.red(),
                    e.red()
                );
                to_remove.push(addr);
            }
        }
    }

    for addr in to_remove {
        clients_map.remove(&addr);
        println!("Removed disconnected client: {}", addr.yellow());
    }

    Ok(())
}
