/// General purpose notifier like a MPSC sender, but with a generic backend.
pub struct Notifier<T>
where
    T: Send,
{
    inner: Box<dyn NotifierImpl<T>>,
}

impl<T> Notifier<T> where T: Send {
    /// Sends the notification through the backend
    pub fn notify(&self, value: T) {
        self.inner.notify(value)
    }
}

impl<T: 'static> Notifier<T>
where
    T: Send,
{
    /// Creates a new notifier from a simple function pointer.
    pub fn from_fn(inner: fn(T)) -> Self {
        Self {
            inner: Box::new(inner),
        }
    }

    /// Creates a new notifier from a custom backend.
    pub fn from_impl<I: 'static>(inner: I) -> Self
    where
        I: NotifierImpl<T>,
    {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl<T> Clone for Notifier<T>
where
    T: Send,
{
    fn clone(&self) -> Self {
        self.inner.replicate()
    }
}

/// Notifier backend, like a MPSC sender.
pub trait NotifierImpl<T>: Send
where
    T: Send,
{
    /// Sends a notification and returns immediately after sending.
    fn notify(&self, value: T);

    /// Clones the implementation.
    fn replicate(&self) -> Notifier<T>;
}

impl<T: 'static> NotifierImpl<T> for fn(value: T)
where
    T: Send,
{
    fn notify(&self, value: T) {
        self(value)
    }

    fn replicate(&self) -> Notifier<T> {
        Notifier::from_fn(*self)
    }
}
