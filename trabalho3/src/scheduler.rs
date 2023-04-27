use std::{
    error::Error,
    io,
    sync::{atomic::AtomicBool, Arc},
};

use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use log::{debug, error, info, warn};

use crate::messages::{Message, MessageType};

pub struct Scheduler {
    queue: Arc<Mutex<Vec<u8>>>,
    listener: TcpListener,
    is_busy: Arc<AtomicBool>,
}

impl Scheduler {
    pub async fn new(addr: String) -> Self {
        info!("[SCHEDULER] Starting scheduler at {}", addr.to_owned());
        Self {
            queue: Arc::new(Mutex::new(vec![])),
            listener: TcpListener::bind(addr)
                .await
                .expect("Could not open TCPListener."),
            is_busy: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(&self) -> ! {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("[SCHEDULER] Starting connection with {}", addr);
                    let queue = Arc::clone(&self.queue);
                    let occupied = Arc::clone(&self.is_busy);
                    tokio::task::spawn(async {
                        handle_connection(stream, queue, occupied).await.unwrap();
                    });
                }
                Err(e) => {
                    error!("[SCHEDULER] Failed to get client: {:?}", e);
                    println!("failed to get client: {:?}", e);
                }
            }
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    queue_arc: Arc<Mutex<Vec<u8>>>,
    occupied_arc: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    loop {
        stream.readable().await?;
        let mut buf = [0; 10];
        match stream.try_read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                debug!("[SCHEDULER] [HANDLER] read {} bytes", n);
                let message = Message::from_bytes(&buf)?;
                info!(
                    "[SCHEDULER] [HANDLER] Message received: {}",
                    message.to_string()
                );
                match message.message_type() {
                    MessageType::Request => {
                        let grant_message = Message::new(0, MessageType::Grant);
                        let occupied = Arc::clone(&occupied_arc);
                        info!(
                            "[SCHEDULER] [HANDLER] Waiting for availability on process #{}",
                            message.sender
                        );
                        let mutex = Arc::clone(&queue_arc);
                        let mut connections = mutex.lock().await;
                        connections.push(message.sender);
                        while occupied.swap(true, std::sync::atomic::Ordering::SeqCst) {}
                        drop(connections);
                        info!(
                            "[SCHEDULER] [HANDLER] Granting access to process #{}.",
                            message.sender
                        );
                        stream.try_write(&grant_message.to_bytes()?)?;
                        info!(
                            "[SCHEDULER] [HANDLER] Access to process #{} was granted.",
                            message.sender
                        );
                    }
                    MessageType::Release => {
                        info!(
                            "[SCHEDULER] [HANDLER] Releasing access to process #{}.",
                            message.sender
                        );
                        let occupied = Arc::clone(&occupied_arc);
                        occupied.swap(false, std::sync::atomic::Ordering::SeqCst);
                        let mutex = Arc::clone(&queue_arc);
                        let mut connections = mutex.lock().await;
                        *connections = connections[1..].to_vec();
                        info!(
                            "[SCHEDULER] [HANDLER] Process #{} released.",
                            message.sender
                        );
                    }
                    _ => break,
                }
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(ref e) if e.kind() == io::ErrorKind::ConnectionAborted => {
                info!("[SCHEDULER] Connection was aborted");
                continue;
            }
            Err(e) => {
                error!("[SCHEDULER] [HANDLER]: {}", e);
                return Err(e.into());
            }
        }
    }
    Ok(())
}
