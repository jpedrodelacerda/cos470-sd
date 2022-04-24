use sd::consumer::Consumer;

fn main() {
    let mut consumer =
        Consumer::from_socket("127.0.0.1:31337".to_string()).expect("failed to create consumer");
    let _ = consumer.read().unwrap();
}
