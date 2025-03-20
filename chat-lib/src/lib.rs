use anyhow::{anyhow, Context, Result};
use bincode;
use serde::{Deserialize, Serialize};
use yansi::Paint;

use std::fs::create_dir;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File { name: String, content: Vec<u8> },
}

impl MessageType {
    /*
     * It uses the first part of the message to determine wheather it is a command to send a file or an image.
     * Or if the user wants to quit or just plain send a text.
     */
    pub fn determine_outgoing_message(msg: &String) -> Result<Self> {
        match msg.as_str() {
            ".quit" => {
                println!("{}", "Good bye!".green());
                exit(0)
            }
            _ if msg.starts_with(".file") => {
                let path = Self::extract_path(&msg)?;

                if let Some(file_name) = Self::extract_file_name(&path) {
                    let file_content = Self::serialize_file(&path)?;

                    Ok(Self::File {
                        name: file_name.to_string(),
                        content: file_content,
                    })
                } else {
                    Err(anyhow!("File path is wrong...".red()))
                }
            }
            _ if msg.starts_with(".image") => {
                let path = Self::extract_path(&msg)?;

                let image_bin = Self::serialize_file(&path)?;
                Ok(Self::Image(image_bin))
            }
            _ => {
                let text_message: String = msg.to_string();
                Ok(Self::Text(text_message))
            }
        }
    }

    fn extract_file_name<P: AsRef<Path>>(path: &P) -> Option<&str> {
        Path::new(path.as_ref()).file_name()?.to_str()
    }

    fn extract_path(command: &String) -> Result<PathBuf> {
        let command_parts: Vec<&str> = command.splitn(2, " ").collect();
        PathBuf::from_str(&command_parts[1]).context("invalid path".red())
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let serialized = bincode::serialize(&self)?;
        Ok(serialized)
    }

    fn deserialize_from_bytes(input: &Vec<u8>) -> Result<Self> {
        bincode::deserialize(input).map_err(|e| anyhow!("Deserialization error: {}", e.red()))
    }

    /*
     * this function takes in a stream serializes the MessageType into bytes
     * gets the peer addrass from the hashmap and writes in the length then the content (image, file or text in bytes)
     * and finally it flushes the stream, effectively sending it...
     */
    pub fn send_message(&self, stream: &mut TcpStream) -> Result<()> {
        let serialized: Vec<u8> = self.serialize()?;
        let serialized_u8: &[u8] = &serialized;
        let addr = stream
            .peer_addr()
            .map(|a| a.to_string())
            .unwrap_or_else(|_| "<unknown address>".to_string());

        let len = serialized.len() as u32;
        stream
            .write(&len.to_be_bytes())
            .with_context(|| format!("Error writing length bytes to client {}", addr.red()))?;

        stream
            .write_all(serialized_u8)
            .with_context(|| format!("Error writing message to client {}", addr.red()))?;

        stream
            .flush()
            .with_context(|| format!("Error flushing stream to client {}", addr.red()))?;

        Ok(())
    }

    /*
     * To receive a message we first get the length of it in bytes
     * it converts the bytes from big endian to u32 and casts it to usize
     * we create the buffer with the specified length
     * and finally deserialize the content
     */
    pub fn receive_message(mut stream: &TcpStream) -> Result<Self> {
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes)?;

        let len = u32::from_be_bytes(len_bytes) as usize;

        let mut buffer = vec![0u8; len];
        stream.read_exact(&mut buffer)?;
        Self::deserialize_from_bytes(&buffer)
    }

    fn serialize_file(path: &Path) -> Result<Vec<u8>> {
        let mut f = File::open(path)?;
        let mut buffer = Vec::new();

        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn save_file_to_disk(path: String, buf: &Vec<u8>) -> Result<()> {
        let parent_dir = Path::new(&path)
            .parent()
            .context("Something went wrong...".red())?;

        if !parent_dir.exists() {
            create_dir(parent_dir)?;
        }

        let mut file = File::create(path)?;
        file.write_all(&buf)?;
        Ok(())
    }
}
