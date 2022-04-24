use sd::producer::Producer;

fn main() {
    let mut producer =
        Producer::from_socket("127.0.0.1:31337".to_string()).expect("failed to create producer");
    // producer.write(3);
    producer.produce_random_ints(3).unwrap();
}
