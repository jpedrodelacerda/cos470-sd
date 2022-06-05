use std::{
    error::Error,
    io,
    sync::{atomic::AtomicU16, Arc, Mutex},
};

use tokio::net::{TcpListener, TcpStream};

use log::info;

use crate::messages::Message;

pub struct Scheduler {
    queue: Arc<Mutex<Vec<Connection>>>,
    listener: TcpListener,
    process_count: AtomicU16,
}

struct Connection {
    conn: TcpStream,
    id: u8,
}

impl Scheduler {
    pub async fn new(addr: String) -> Self {
        Self {
            queue: Arc::new(Mutex::new(vec![])),
            listener: TcpListener::bind(addr)
                .await
                .expect("Could not open TCPListener."),
            process_count: AtomicU16::new(0),
        }
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    info!("[SCHEDULER] Starting connection with {}", addr);
                    self.handle_connection(stream).await?;
                }
                Err(e) => println!("failed to get client: {:?}", e),
            }
        }
    }

    async fn handle_connection(&mut self, stream: TcpStream) -> Result<(), Box<dyn Error>> {
        loop {
            stream.readable().await?;
            let mut buf = [0; 10];
            match stream.try_read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    println!("read {} bytes", n);
                    let m = Message::from_bytes(&buf)?;
                    info!("{}", m);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }
}
