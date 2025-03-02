use crate::chat::MessageType;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::net::TcpListener;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

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
        match MessageType::receive_message(&stream.try_clone().unwrap()) {
            Ok(message) => match message {
                MessageType::Text(text_content) => {
                    println!("{text_content}");
                    let clients_map = clients.lock().unwrap();
                    if let Some(client_stream) = clients_map.get(&addr) {
                        println!("Connected to: {}", client_stream.peer_addr().unwrap());
                        let msg = MessageType::Text(text_content);
                        drop(clients_map);
                        broadcast(&clients, &addr, msg)?
                    }
                }
                MessageType::Image(image_data) => {
                    let clients_map = clients.lock().unwrap();
                    if let Some(client_stream) = clients_map.get(&addr) {
                        println!("Connected to: {}", client_stream.peer_addr().unwrap());
                        println!("Receiving image...");
                        let file_name = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let path = format!("./images/{file_name}.png");
                        drop(clients_map);
                        MessageType::save_file_to_disk(path, &image_data)?;
                        let msg = MessageType::Image(image_data);
                        broadcast(&clients, &addr, msg)?
                    }
                }
                MessageType::File { name, content } => {
                    let clients_map = clients.lock().unwrap();
                    if let Some(client_stream) = clients_map.get(&addr) {
                        println!("Connected to: {}", client_stream.peer_addr().unwrap());
                        println!("Received file: {}", name);
                        let path = format!("./files/{name}");
                        drop(clients_map);
                        MessageType::save_file_to_disk(path, &content)?;
                        let msg = MessageType::File { name, content };
                        broadcast(&clients, &addr, msg)?
                    }
                }
                MessageType::Empty => {
                    eprintln!("Yea we are not sending empty messages...");
                }
            },
            Err(e) => {
                println!("Error receiving message: {}", e);
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

    for (&addr, stream) in clients_map.iter_mut() {
        if addr == *sender_addr {
            continue;
        }

        match bincode::serialize(&msg) {
            Ok(serialized) => {
                let len = serialized.len() as u32;
                let len_bytes = len.to_be_bytes();

                match stream.write_all(&len_bytes) {
                    Ok(_) => match stream.write_all(&serialized) {
                        Ok(_) => match stream.flush() {
                            Ok(_) => {
                                println!("Successfully sent message to client {}", addr);
                            }
                            Err(e) => {
                                println!("Error flushing stream to {}: {}", addr, e);
                                to_remove.push(addr);
                            }
                        },
                        Err(e) => {
                            println!("Error writing message to client {}: {}", addr, e);
                            to_remove.push(addr);
                        }
                    },
                    Err(e) => {
                        println!("Error writing length bytes to client {}: {}", addr, e);
                        to_remove.push(addr);
                    }
                }
            }
            Err(e) => {
                println!("Error serializing message for client {}: {}", addr, e);
                continue;
            }
        }
    }

    for addr in to_remove {
        clients_map.remove(&addr);
        println!("Removed disconnected client: {}", addr);
    }

    Ok(())
}
