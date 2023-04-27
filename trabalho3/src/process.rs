use std::{
    error::Error,
    io::{self, Write},
};

use log::{debug, error, info, warn};
use std::fs::OpenOptions;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::messages::{Message, MessageType};

pub struct Process {
    id: u8,
    stream: TcpStream,
    write_to: String,
    sleep_time: u64,
    repeat: u32,
    counter: u32,
}

impl Process {
    pub async fn new(
        id: u8,
        address: String,
        write_to: String,
        sleep_time: u64,
        repeat: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let stream = tokio::net::TcpStream::connect(address).await?;
        info!("[PROCESS] [#{}] Connected successfully", id);
        Ok(Process {
            id,
            stream,
            write_to,
            sleep_time,
            repeat,
            counter: 0,
        })
    }

    pub async fn send_message(&mut self, message_type: MessageType) -> Result<(), Box<dyn Error>> {
        let message = Message::new(self.id, message_type);
        let message_bytes = message.to_bytes()?;
        self.stream.write(&message_bytes).await?;
        Ok(())
    }

    pub async fn handle_connection(&mut self) -> Result<(), Box<dyn Error>> {
        debug!("[PROCESS] [HANDLER] Connection from #{}", self.id);
        while self.counter < self.repeat {
            self.send_message(MessageType::Request).await?;
            self.stream.readable().await?;
            let mut buf: [u8; 10] = [0; 10];
            match self.stream.try_read(&mut buf) {
                Ok(0) => continue,
                Ok(n) => {
                    info!("[PROCESS] [READING] received {} bytes", n);
                    let message = Message::from_bytes(&buf)?;
                    match message.message_type {
                        MessageType::Grant => {
                            info!("Request was granted for process #{}", self.id);
                            self.log_result().await?;
                        }
                        msg => {
                            warn!("[PROCESS] [HANDLER] Received given message: {}", msg)
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => {
                    info!("Connection was closed. Finishing process #{}", self.id);
                    return Ok(());
                }
                Err(e) => {
                    error!("Error handling connection on process #{}: {}", self.id, e);
                    return Err(e.into());
                }
            }
        }
        info!("[PROCESS] Finishng task #{}", self.id);
        Ok(())
    }

    async fn log_result(&mut self) -> Result<(), Box<dyn Error>> {
        let path = self.write_to.as_str();
        info!(
            "[PROCESS] [RESULT] Logging result at {} from #{}",
            path, self.id
        );
        let mut file = OpenOptions::new().write(true).append(true).open(path)?;
        let now = chrono::Local::now();
        let log_record = format!("{} - Writing from {}\n", now, self.id);
        file.write(log_record.as_bytes())?;
        info!(
            "[PROCESS] [RESULT] Result logged successfully from #{}",
            self.id
        );
        info!("[PROCESS] #{} is sleeping", self.id);
        tokio::time::sleep(tokio::time::Duration::from_millis(self.sleep_time)).await;
        info!("[PROCESS] [HANDLER] Releasing from #{}", self.id);
        self.stream
            .try_write(&Message::new(self.id, MessageType::Release).to_bytes()?)?;
        info!("[PROCESS] [HANDLER] #{} released successfully", self.id);
        self.counter += 1;
        Ok(())
    }
}
