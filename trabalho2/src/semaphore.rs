use std::sync::{Condvar, Mutex};

#[derive(Debug)]
pub struct Semaphore {
    counter: Mutex<usize>,
    cvar: Condvar,
}

impl Semaphore {
    pub fn new(count: usize) -> Self {
        Semaphore {
            counter: Mutex::new(count),
            cvar: Condvar::new(),
        }
    }

    pub fn available_threads(&self) -> usize {
        let count = self.counter.lock().unwrap();
        *count
    }

    pub fn wait(&self) {
        let mut count = self.counter.lock().unwrap();
        while *count <= 0 {
            count = self.cvar.wait(count).unwrap();
        }
        *count -= 1;
    }

    pub fn signal(&self) {
        let mut count = self.counter.lock().unwrap();
        *count += 1;
        self.cvar.notify_one();
    }
}
