use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

#[derive(Default)]
pub struct TokenGenerator {
    prefix: String,
    counter: AtomicU32,
}

impl TokenGenerator {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            counter: AtomicU32::new(0),
        }
    }

    pub fn reset(&self) {
        self.counter.store(0, Relaxed);
    }

    pub fn generate(&self) -> String {
        let cur = self.counter.load(Relaxed);
        self.counter.store(cur + 1, Relaxed);
        self.prefix.to_string() + &cur.to_string()
    }

    // pub fn roll_back(&self) {
    //     let cur = self.counter.load(Relaxed);
    //     assert_ne!(cur, 0);
    //     self.counter.store(cur - 1, Relaxed);
    // }
}