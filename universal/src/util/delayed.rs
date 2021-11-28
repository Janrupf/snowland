use std::borrow::Borrow;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Debug)]
pub struct Delayed<T>
where
    T: Send,
{
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T> Delayed<T>
where
    T: Send,
{
    /// Creates a new delayed and its associated resolver.
    pub fn new() -> (Self, DelayedResolver<T>) {
        let inner = (Mutex::new(None), Condvar::new());

        let inner = Arc::new(inner);
        let resolver_inner = inner.clone();

        (
            Self { inner },
            DelayedResolver {
                inner: resolver_inner,
            },
        )
    }

    /// Waits for the value to be set.
    pub fn wait(self) -> T {
        let (mutex, var) = &*self.inner;

        let mut guard = mutex.lock().unwrap();
        while guard.borrow().is_none() {
            guard = var.wait(guard).unwrap();
        }

        guard.take().unwrap()
    }
}

#[derive(Debug)]
pub struct DelayedResolver<T>
where
    T: Send,
{
    inner: Arc<(Mutex<Option<T>>, Condvar)>,
}

impl<T> DelayedResolver<T>
where
    T: Send,
{
    /// Resolves the delayed value.
    pub fn resolve(self, value: T) {
        let (mutex, var) = &*self.inner;

        let mut guard = mutex.lock().unwrap();
        guard.replace(value);

        var.notify_one();
    }
}
