use nativeshell::codec::{MethodCall, MethodCallReply, Value};
use nativeshell::shell::{ContextRef, EngineHandle, MethodCallHandler, MethodChannel};

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
        reply.send_error(
            "NOT_IMPLEMENTED",
            Some(&format!("Method {} is not implemented", call.method)),
            Value::Null,
        );
    }
}
