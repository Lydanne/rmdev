use std::sync::atomic::AtomicUsize;

#[derive(Debug, Default)]
pub struct Signal {
    code: AtomicUsize,
}

impl Signal {
    pub fn new() -> Self {
        Signal {
            code: AtomicUsize::new(0),
        }
    }

    pub fn set(&self, code: usize) {
        self.code.store(code, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get(&self) -> usize {
        self.code.load(std::sync::atomic::Ordering::Relaxed)
    }
}
