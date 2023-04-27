use sd::{messages::MessageType, process::Process};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut handles = vec![];
    for n in 1..3 {
        handles.push(tokio::task::spawn(async move {
            let mut proc = Process::new(
                n,
                "0.0.0.0:1337".to_string(),
                "/tmp/sd.log".to_string(),
                2000,
                10,
            )
            .await
            .unwrap();
            proc.handle_connection()
                .await
                .expect("failed to handle connection");
        }));
    }
    for handle in handles {
        handle.await?;
    }
    Ok(())
}
