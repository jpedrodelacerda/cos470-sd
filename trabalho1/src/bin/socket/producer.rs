use sd::producer::Producer;
use std::env;
use std::str::FromStr;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Please specify how many numbers to generate.");
        std::process::exit(1);
    }
    let number_count =
        i32::from_str(&args[1]).expect("failed to parse how many numbers should be produced.");

    let mut producer =
        Producer::from_socket("127.0.0.1:31337".to_string()).expect("failed to create producer");
    producer.produce_random_ints(number_count).unwrap();
}
