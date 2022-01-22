use nativeshell::codec::{MethodCall, MethodCallReply, Value};
use serde::Serialize;

pub struct Responder {
    inner: MethodCallReply<Value>,
    method: String,
}

impl Responder {
    pub fn new(call: &MethodCall<Value>, inner: MethodCallReply<Value>) -> Self {
        Self {
            inner,
            method: call.method.clone(),
        }
    }

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
