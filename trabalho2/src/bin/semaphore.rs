use log::{debug, info};
use rand::prelude::*;
use sd::semaphore::Semaphore;
use std::fs::File;
use std::io::prelude::*;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

fn main() {
    // Set up env_logger so we can track what is happening
    // using environment variable RUST_LOG=debug
    env_logger::init();

    // First, we will set the counter of consumed integers that
    // will be shared across the threads. We do not need an ARC
    // in this case since we will be using an atomic integer type.
    static CONSUMER_COUNTER: AtomicI32 = AtomicI32::new(0);

    const CONSUMER_LIMIT: i32 = 100_000;

    // We can set up the cases in an array. Each case represents the number
    // of consumers and producers, respectively, e.g. (n_c, n_p).
    let cases = [(1, 1), (1, 2), (1, 4), (1, 8), (2, 1), (4, 1), (8, 1)];

    // We will keep our records inside a vector and print it at the end of our run.
    let mut results: Vec<String> = vec![];
    let results_header = "n,n_p,n_c,time".to_string();
    results.push(results_header);

    // We can keep track of our buffer length with
    // an array and create the buffer on each loop.
    let vector_lengths = [1, 2, 4, 16, 32];
    for (n_p, n_c) in cases {
        for n in vector_lengths {
            // We need will be using Semaphores to control
            // how many consumers and producers should be running.
            // We will also be using an ARC to share the semaphores
            // between threads.
            let empty_semaphore = Arc::new(Semaphore::new(n));
            let full_semaphore = Arc::new(Semaphore::new(0));

            for _ in 0..10 {
                info!("[CASE] STARTING NEW CASE: n = {n}, n_p = {n_p}, n_c = {n_c}");
                // We will be using a Mutex to ensure
                // that only a single thread has access to the buffer
                // and this will also enable us to pass the buffer inside
                // an ARC to be shared between threads.
                let vector_mutex: Arc<Mutex<Vec<i32>>> = Arc::new(Mutex::new(vec![0; n as usize]));

                // We also need to have control over our threads.
                // We can get that by creating a handler buffer so
                // that we can wait all of them return.
                let mut handles: Vec<JoinHandle<()>> = vec![];

                // We can start keeping track of our time here.

                let start = std::time::Instant::now();

                // We will set up a handle buffer to handle both
                // consumer and producer threads.
                // let mut handles: Vec<JoinHandle<()>> = vec![];
                // We need to clone the ARC of resources to make them
                // available for our threads.
                for _p in 0..n_p {
                    debug!("[PRODUCER][{_p}] Starting");
                    let producer_empty_semaphore = Arc::clone(&empty_semaphore);
                    let producer_full_semaphore = Arc::clone(&full_semaphore);
                    let producer_vector_mutex = Arc::clone(&vector_mutex);
                    // We initialize a Producer thread that produces indefinitely.
                    thread::spawn(move || {
                        // Initialize random number generator
                        let mut rng = rand::thread_rng();
                        loop {
                            // We need to clone both semaphores and the vector inside the ARCs.
                            let empty = Arc::clone(&producer_empty_semaphore);
                            let full = Arc::clone(&producer_full_semaphore);
                            let vector_arc = Arc::clone(&producer_vector_mutex);
                            // We need to make sure that we can take temporary ownership
                            // of the vector. We only do this after we have the semaphores
                            // with enough space.
                            debug!("[PRODUCER][{_p}] Acquiring EMPTY");
                            empty.wait();
                            debug!("[PRODUCER][{_p}] EMPTY acquired successfully.");
                            // Now we can lock the buffer Mutex.
                            debug!("[PRODUCER][{_p}] Locking VECTOR_MUTEX.");
                            let mut vector = vector_arc.lock().unwrap();
                            debug!("[PRODUCER][{_p}] VECTOR_MUTEX locked successfully.");
                            for i in 0..vector.len() {
                                // If the position contains a zero, we fill the spot
                                // with the current index.
                                if vector[i] == 0 {
                                    debug!("[PRODUCER][{_p}] Found empty space in {i}");
                                    // We can now fill the empty space with the random number.
                                    vector[i] = rng.gen_range(1..10_000_001);
                                    debug!(
                                        "[PRODUCER][{_p}] Filled position {i} with {}",
                                        vector[i]
                                    );
                                    break;
                                }
                            }
                            // We now drop the vector reference so that the buffer
                            // Mutex is unlocked.
                            debug!("[PRODUCER][{_p}] Unlocking VECTOR_MUTEX.");
                            drop(vector);
                            debug!("[PRODUCER][{_p}] VECTOR_MUTEX unlocked successfully.");
                            // We now use the release method to signal that there is
                            // an empty space inside the buffer.
                            debug!("[PRODUCER][{_p}] Releasing FULL.");
                            full.signal();
                            debug!("[PRODUCER][{_p}] FULL released successfully.");
                        }
                    });
                }

                for _c in 0..n_c {
                    let consumer_empty_semaphore = Arc::clone(&empty_semaphore);
                    let consumer_full_semaphore = Arc::clone(&full_semaphore);
                    let consumer_vector_mutex = Arc::clone(&vector_mutex);

                    // Spawn consumer thread and register the handle on handles buffer.
                    handles.push(thread::spawn(move || {
                        debug!("[CONSUMER][{_c}] Starting.");
                        while CONSUMER_COUNTER.load(Ordering::SeqCst) < CONSUMER_LIMIT {
                            // We need to clone both semaphores and the vector inside the ARCs.
                            let empty = Arc::clone(&consumer_empty_semaphore);
                            let full = Arc::clone(&consumer_full_semaphore);
                            let vector_arc = Arc::clone(&consumer_vector_mutex);
                            // We need to make sure that we can take temporary ownership
                            // of the vector. We only do this after we have the semaphores
                            // with enough space.
                            debug!("[CONSUMER][{_c}] Acquiring FULL");
                            full.wait();
                            debug!("[CONSUMER][{_c}] FULL acquired successfully");
                            // Now we can lock the buffer Mutex.
                            debug!("[CONSUMER][{_c}] Locking VECTOR_MUTEX");
                            let mut vector = vector_arc.lock().unwrap();
                            debug!("[CONSUMER][{_c}] VECTOR_MUTEX locked successfully");
                            // let mut to_consume = -1;
                            for i in 0..vector.len() {
                                // If the position does not contain a zero
                                // we consume it. This means we will process it
                                // and fill the space with a zero.
                                if vector[i] != 0 {
                                    debug!(
                                        "[CONSUMER][{_c}] Consuming {} at position {}",
                                        vector[i], i
                                    );
                                    is_prime(vector[i]);
                                    debug!("[CONSUMER][{_c}] Consumed {} successfully", vector[i]);
                                    debug!("[CONSUMER][{_c}] Emptying position {}.", i);
                                    vector[i] = 0;
                                    if vector[i] == 0 {
                                        debug!("[CONSUMER][{_c}] Position {i} is now empty");
                                    }
                                    // CONSUMER_COUNTER.fetch_add/* ( */1, Ordering::Acquire);
                                    debug!("[CONSUMER][{_c}] Updating CONSUMER_COUNTER.");
                                    CONSUMER_COUNTER
                                        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                                            Some(x + 1)
                                        })
                                        .unwrap();
                                    debug!(
                                        "[CONSUMER][{_c}] CONSUMER_COUNTER = {}",
                                        CONSUMER_COUNTER.load(Ordering::SeqCst)
                                    );
                                    break;
                                }
                            }
                            // We now drop the vector reference so that the buffer
                            // Mutex is unlocked.
                            debug!("[CONSUMER][{_c}] Unlocking VECTOR_MUTEX");
                            drop(vector);
                            debug!("[CONSUMER][{_c}] VECTOR_MUTEX was unlocked successfully");
                            // We now use the release method to signal that there is
                            // an empty space inside the buffer.
                            debug!("[CONSUMER][{_c}] Releasing EMPTY");
                            empty.signal();
                            debug!("[CONSUMER][{_c}] EMPTY released successfully");
                        }
                        debug!("[CONSUMER][{_c}] Finishing.");
                    }));
                }

                // Wait for all threads to finish.
                for handle in handles {
                    handle.join().unwrap();
                }
                // Loop until we hit the CONSUMER_LIMIT.
                // while CONSUMER_COUNTER.load(Ordering::SeqCst) < consumer_limit {}
                // When the loop is over, we can consider our finishing time.
                let elapsed = start.elapsed();

                // We can register the results and reset the consumer counter.
                let result = format!("{},{},{},{}", n, n_p, n_c, elapsed.as_millis());
                info!("{result}");
                results.push(result);

                // We need to reset the CONSUMER_COUNTER.
                CONSUMER_COUNTER.store(0, Ordering::SeqCst);
            }
        }
    }

    // We will write results to a file.
    let mut file = File::create("./data/semaphore_results.csv")
        .expect("Could not open '/.data/semaphore_results.csv'.");
    for entry in results.into_iter().skip(1) {
        println!("Result: {}", entry);
        file.write_all(format!("{entry}\n").as_bytes()).unwrap();
    }
}

// Helper function for consumer.
fn is_prime(int: i32) -> bool {
    if int == 1 {
        println!("{} is not prime", int);
        return false;
    }
    let sqrt = (int as f32).sqrt() as i32;
    for i in 2..sqrt {
        if int % i == 0 {
            println!("{} is not prime", int);
            return false;
        }
    }
    println!("{} is prime", int);
    return true;
}
