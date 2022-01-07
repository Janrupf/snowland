use nativeshell::codec::{MethodCall, MethodCallReply, Value};
use nativeshell::shell::{ContextRef, EngineHandle, MethodCallHandler, MethodChannel};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub struct TestCommunicationChannel;

impl TestCommunicationChannel {
    pub fn register(context: &ContextRef) -> MethodChannel {
        let instance = Self {};
        MethodChannel::new(context.weak(), "test_channel", instance)
    }
}

impl MethodCallHandler for TestCommunicationChannel {
    fn on_method_call(
        &mut self,
        call: MethodCall<Value>,
        reply: MethodCallReply<Value>,
        engine: EngineHandle,
    ) {
        match call.method.as_str() {
            "test" => {
                let messages: Vec<String> = match reserialize(call.args) {
                    Ok(v) => v,
                    Err(err) => {
                        reply.send_error("INVALID_ARGS", Some(&err.to_string()), Value::Null);

                        return;
                    }
                };

                log::debug!("Received args: {:#?}", messages);
                reply.send_ok(Value::Null);
            }
            _ => {
                reply.send_error(
                    "NOT_IMPLEMENTED",
                    Some(&format!("Method {} is not implemented", call.method)),
                    Value::String(call.method),
                );
                return;
            }
        }
    }
}

fn reserialize<'a, I: Serialize, O: Deserialize<'a>>(input: I) -> Result<O, ReserializeError> {
    let immediate = serde_json::to_value(input).map_err(ReserializeError::SerializationFailed)?;
    let output = O::deserialize(immediate).map_err(ReserializeError::DeserializationFailed)?;

    Ok(output)
}

#[derive(Debug, Error)]
enum ReserializeError {
    #[error("failed to serialize value: {0}")]
    SerializationFailed(serde_json::Error),

    #[error("failed to deserialize value: {0}")]
    DeserializationFailed(serde_json::Error),
}
