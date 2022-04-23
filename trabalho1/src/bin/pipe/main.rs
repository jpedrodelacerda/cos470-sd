use std::env;
use std::str::FromStr;

use nix::unistd::pipe;
use nix::unistd::{fork, ForkResult, Pid};
use sd::consumer::Consumer;
use sd::producer::Producer;

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Please specify how many numbers to generate.");
        std::process::exit(1);
    }
    let number_count =
        i32::from_str(&args[1]).expect("failed to parse how many numbers should be produced.");

    let (stdin, stdout) = pipe().expect("failed to create pipe");

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            println!(
                "continuing execution in parent process. Parent pid is {} and child pid is {}",
                Pid::this(),
                child
            );
            let mut producer = Producer::from_fd(stdout).expect("failed to create producer");
            producer
                .produce_random_ints(number_count)
                .expect("failed to produce ints");
        }
        Ok(ForkResult::Child) => {
            let mut consumer = Consumer::from_fd(stdin).expect("failed to create consumer");
            loop {
                consumer.read().expect("failed to read");
            }
        }
        Err(_) => println!("failed to fork"),
    }
}
