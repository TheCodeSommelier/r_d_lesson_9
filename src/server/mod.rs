use anyhow::{anyhow, Error, Result};
use bincode::serialize;
use regex::Regex;
use std::collections::HashMap;
use std::fs::create_dir;
use std::io::Write;
use std::net::TcpListener;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::chat::MessageType;

pub fn validate_port(port: &String) -> Result<String, anyhow::Error> {
    let regex = Regex::new(r"[0-9]")?;

    if regex.is_match(port) {
        Ok(port.clone())
    } else if port == "" {
        Ok("11111".to_string())
    } else {
        Err(anyhow!("The port is invalid..."))
    }
}

pub fn validate_host(host: &String) -> Result<String, anyhow::Error> {
    let regex = Regex::new(
        r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$",
    )?;

    if regex.is_match(host) {
        Ok(host.clone())
    } else if host == "" {
        Ok("127.0.0.1".to_string())
    } else {
        Err(anyhow!("The host is invalid..."))
    }
}

fn dir_exists(path: &str) -> bool {
    Path::new(path).exists()
}

pub fn listen_and_accept(address: &String) -> Result<()> {
    let listener = TcpListener::bind(address)?;
    let clients = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming() {
        let Ok(stream) = stream else {
            continue;
        };
        let Ok(addr) = stream.peer_addr() else {
            continue;
        };

        println!("New client connected: {}", addr);

        let clients_clone = Arc::clone(&clients);

        {
            let mut clients_map = clients.lock().unwrap();
            clients_map.insert(addr, stream.try_clone()?);
        }

        // Spawn a thread to handle this client
        thread::spawn(move || {
            if let Err(e) = handle_client(stream, addr, clients_clone) {
                println!("Client error: {}", e);
            }
        });
    }

    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
) -> Result<()> {
    loop {
        match MessageType::receive_message(stream.try_clone().unwrap()) {
            Ok(message) => match message {
                MessageType::Text(text_content) => {
                    println!("{text_content}");
                    if let Some(stream) = clients.get(&addr) {
                        println!("Connected to: {}", stream.peer_addr().unwrap());
                        let text_bin = serialize(&text_content)?;
                        broadcast(&text_bin, &clients, &addr)?
                    }
                }
                MessageType::Image(image_data) => {
                    if let Some(stream) = clients.get(&addr) {
                        println!("Connected to: {}", stream.peer_addr().unwrap());
                        println!("Receiving image...");
                        if !dir_exists("./images") {
                            create_dir("./images")?;
                        }
                        broadcast(&image_data, &clients, &addr)?
                    }
                }
                MessageType::File { name, content } => {
                    if let Some(stream) = clients.get(&addr) {
                        println!("Connected to: {}", stream.peer_addr().unwrap());
                        println!("Received file: {}", name);
                        if dir_exists("./files") {
                            create_dir("./files")?;
                        }
                        broadcast(&content, &clients, &addr)?
                    }
                }
            },
            Err(e) => {
                println!("Error receiving message: {}", e);
                // Handle error
            }
        }
    }

    Ok(())
}

fn broadcast(
    message: &Vec<u8>,
    clients: &Arc<Mutex<HashMap<SocketAddr, TcpStream>>>,
    sender_addr: &SocketAddr,
) -> Result<()> {
    let mut to_remove = Vec::new();
    let mut clients_map = clients.lock().unwrap();

    for (&addr, stream) in clients_map.iter_mut() {
        match stream.write_all(message) {
            Ok(_) => match stream.flush() {
                Ok(_) => {}
                Err(e) => {
                    println!("Error flushing stream to {}: {}", addr, e);
                    to_remove.push(addr);
                }
            },
            Err(e) => {
                println!("Error writing to client {}: {}", addr, e);
                to_remove.push(addr);
            }
        }
    }

    for addr in to_remove {
        clients_map.remove(&addr);
        println!("Removed disconnected client: {}", addr);
    }

    Ok(())
}
