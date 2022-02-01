use nativeshell::codec::{MethodCall, MethodCallReply, Value};
use nativeshell::shell::RunLoopSender;
use nativeshell::util::Capsule;
use serde::Serialize;

pub trait InnerResponder
where
    Self: Sized,
{
    fn send_ok(self, value: Value);

    fn send_error(self, code: &str, message: Option<&str>, details: Value);
}

pub struct DirectInnerResponder {
    inner: MethodCallReply<Value>,
}

impl DirectInnerResponder {
    pub fn new(inner: MethodCallReply<Value>) -> Self {
        Self { inner }
    }
}

impl InnerResponder for DirectInnerResponder {
    fn send_ok(self, value: Value) {
        self.inner.send_ok(value);
    }

    fn send_error(self, code: &str, message: Option<&str>, details: Value) {
        self.inner.send_error(code, message, details);
    }
}

pub struct ThreadSafeInnerResponder {
    inner: Capsule<MethodCallReply<Value>>,
    sender: RunLoopSender,
}

impl ThreadSafeInnerResponder {
    pub fn new(inner: MethodCallReply<Value>, sender: RunLoopSender) -> Self {
        Self {
            inner: Capsule::new_with_sender(inner, sender.clone()),
            sender,
        }
    }
}

impl ThreadSafeInnerResponder {
    fn with_reply<F>(self, callback: F)
    where
        F: FnOnce(MethodCallReply<Value>) + 'static + Send,
    {
        let Self { mut inner, sender } = self;

        sender.send(move || {
            let reply = inner.take().unwrap();
            callback(reply);
        });
    }
}

impl InnerResponder for ThreadSafeInnerResponder {
    fn send_ok(self, value: Value) {
        self.with_reply(move |r| r.send_ok(value));
    }

    fn send_error(self, code: &str, message: Option<&str>, details: Value) {
        let code = code.to_owned();
        let message = message.map(|s| s.to_owned());

        self.with_reply(move |r| {
            r.send_error(&code, message.as_ref().map(|s| s.as_str()), details);
        });
    }
}

#[must_use = "Dropping a responder without using it hangs a future in the dart VM"]
pub struct Responder<I: InnerResponder> {
    inner: I,
    method: String,
}

impl Responder<DirectInnerResponder> {
    pub fn new(call: &MethodCall<Value>, reply: MethodCallReply<Value>) -> Self {
        Self {
            inner: DirectInnerResponder::new(reply),
            method: call.method.clone(),
        }
    }
}

impl Responder<ThreadSafeInnerResponder> {
    pub fn new(
        call: &MethodCall<Value>,
        reply: MethodCallReply<Value>,
        sender: RunLoopSender,
    ) -> Self {
        Self {
            inner: ThreadSafeInnerResponder::new(reply, sender),
            method: call.method.clone(),
        }
    }
}

impl<I: InnerResponder> Responder<I> {
    pub fn ok<V>(self, v: V)
    where
        V: Serialize,
    {
        match crate::util::reserialize(v) {
            Ok(v) => self.inner.send_ok(v),
            Err(err) => self.inner.send_error(
                "INVALID_RETURN_VALUE",
                Some(&format!(
                    "Failed to convert the return value of method {}: {}",
                    &self.method, err
                )),
                ::nativeshell::codec::Value::String(self.method),
            ),
        };
    }

    pub fn failed(
        self,
        code: impl AsRef<str>,
        message: Option<impl AsRef<str>>,
        details: Option<Value>,
    ) {
        let code = code.as_ref();
        let message = message.as_ref().map(|v| v.as_ref());
        let details = details.unwrap_or(Value::Null);

        self.inner.send_error(code, message, details);
    }

    pub fn result<V, E>(self, res: Result<V, E>)
    where
        V: Serialize,
        E: std::error::Error,
    {
        match res {
            Ok(v) => self.ok(v),
            Err(err) => {
                self.failed("EXECUTION_FAILED", Some(&err.to_string()), None);
            }
        }
    }
}
