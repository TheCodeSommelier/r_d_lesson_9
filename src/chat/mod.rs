use anyhow::{anyhow, Result};
use bincode;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::exit;

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File { name: String, content: Vec<u8> },
}

impl MessageType {
    pub fn determine_outgoing_message(msg: &String) -> Result<Self, anyhow::Error> {
        match msg.as_str() {
            ".quit" => {
                println!("Good bye!");
                exit(200)
            }
            _ if msg.starts_with(".file") => {
                let path = Self::extract_path(&msg);

                if let Some(file_name) = Self::extract_file_name(&path) {
                    let file_content = Self::serialize_file(&path)?;

                    Ok(Self::File {
                        name: file_name.to_string(),
                        content: file_content,
                    })
                } else {
                    Err(anyhow!("File path is wrong..."))
                }
            }
            _ if msg.starts_with(".image") => {
                let path = Self::extract_path(&msg);
                let image_bin = Self::serialize_file(&path)?;
                Ok(Self::Image(image_bin))
            }
            _ => {
                let text_message: String = msg.to_string();
                Ok(Self::Text(text_message))
            }
        }
    }

    fn extract_file_name(path: &String) -> Option<&str> {
        Path::new(path).file_name()?.to_str()
    }

    fn extract_path(command: &String) -> String {
        let command_parts: Vec<&str> = command.splitn(2, " ").collect();
        command_parts[1].to_string()
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let serialized = bincode::serialize(&self)?;
        Ok(serialized)
    }

    fn deserialize_from_bytes(input: &Vec<u8>) -> Result<Self> {
        bincode::deserialize(input).map_err(|e| anyhow!("Deserialization error: {}", e))
    }

    pub fn send_message(self, stream: &mut TcpStream ) -> Result<()> {
        let serialized: Vec<u8> = self.serialize()?;
        let serialized_u8: &[u8] = &serialized;

        let len = serialized.len() as u32;
        stream.write(&len.to_be_bytes())?;

        stream.write_all(serialized_u8)?;

        Ok(())
    }

    pub fn receive_message(mut stream: TcpStream) -> Result<Self> {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes)?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer)?;

        Self::deserialize_from_bytes(&buffer)
    }

    fn serialize_file(path: &str) -> Result<Vec<u8>> {
        let mut f = File::open(path)?;
        let mut buffer = Vec::new();

        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}
