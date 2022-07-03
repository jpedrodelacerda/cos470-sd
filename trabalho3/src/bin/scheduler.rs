use sd::scheduler::Scheduler;

#[tokio::main]
async fn main() -> Result<(), ()> {
    env_logger::init();
    let mut scheduler = Scheduler::new("0.0.0.0:1337".to_string()).await;
    scheduler.start().await;
    Ok(())
}
