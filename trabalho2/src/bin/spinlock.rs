use rand::prelude::*;
use sd::spinlock::Spinlock;
use std::sync::Arc;
use std::thread;

fn main() {
    let k_vals = [1, 2, 4, 8, 16, 32, 64, 128, 256];
    let n_vals = [10_i32.pow(7), 10_i32.pow(8), 10_i32.pow(9)];
    for n in n_vals {
        let vector: Arc<Vec<i8>> = Arc::new(populate_random_i8(n as usize));
        for k in k_vals {
            for _ in 0..10 {
                let vec = Arc::clone(&vector);
                run_case(k, n, &*vec);
            }
        }
    }
}

fn run_case(k: i32, n: i32, random_vector: &Vec<i8>) {
    let step_size: usize = (n / k) as usize;
    let sum = Arc::new(Spinlock::new(0));

    let now = std::time::Instant::now();

    // let vec = Arc::clone(random_vector);
    let mut handles = vec![];
    for chunks in random_vector.chunks(step_size) {
        let sum = Arc::clone(&sum);
        let chunk = chunks.to_owned();
        let handle = thread::spawn(move || {
            let local_sum = sum_inside_thread(&chunk);
            sum.acquire();
            sum.set(*sum.get() + local_sum).unwrap();
            sum.release();
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = now.elapsed();
    println!("{},{},{:?},{}", k, n, sum.get(), elapsed.as_millis());
}

fn sum_inside_thread(vector: &[i8]) -> i32 {
    let mut local_sum: i32 = 0;
    for i in 0..vector.len() {
        let value_as_i32 = vector[i] as i32;
        local_sum += value_as_i32;
    }
    local_sum
}

fn populate_random_i8(size: usize) -> Vec<i8> {
    let mut vector = vec![0; size];
    let mut rng = rand::thread_rng();
    for i in 0..vector.len() {
        vector[i] = rng.gen_range(-100..101);
    }
    vector.to_vec()
}
