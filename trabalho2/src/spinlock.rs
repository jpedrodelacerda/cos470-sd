use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Spinlock {
    locked: AtomicBool,
    data: UnsafeCell<i32>,
}

impl Spinlock {
    pub fn new(initial_value: i32) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(initial_value),
        }
    }

    pub fn acquire(&self) {
        while self.locked.swap(true, Ordering::SeqCst) {}
    }

    pub fn release(&self) {
        self.locked.store(false, Ordering::SeqCst);
    }

    pub fn get(&self) -> &mut i32 {
        unsafe { &mut *self.data.get() }
    }
    pub fn get_mut(&mut self) -> &mut i32 {
        self.data.get_mut()
    }

    pub fn set(&self, int: i32) -> Result<(), ()> {
        if self.locked.load(Ordering::SeqCst) {
            let d = self.data.get();
            unsafe {
                *d = int;
            }
            return Ok(());
        } else {
            return Err(());
        }
    }
}

unsafe impl Sync for Spinlock {}
unsafe impl Send for Spinlock {}

impl std::ops::Deref for Spinlock {
    type Target = i32;

    fn deref(&self) -> &i32 {
        unsafe { &*self.data.get() }
    }
}

impl std::ops::DerefMut for Spinlock {
    fn deref_mut(&mut self) -> &mut i32 {
        unsafe { &mut *self.data.get() }
    }
}
