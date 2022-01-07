use crate::mcr;
use nativeshell::shell::{ContextRef, MethodChannel};

pub struct TestCommunicationChannel;

impl TestCommunicationChannel {
    pub fn register(context: &ContextRef) -> MethodChannel {
        let instance = Self {};
        MethodChannel::new(context.weak(), "test_channel", instance)
    }
}

#[mcr::method_channel_call_handler]
impl TestCommunicationChannel {
    pub fn greet(name: String, input: f64) -> Result<String, std::convert::Infallible> {
        Ok(format!("Hello {}, here is your data: {:?}", name, input))
    }
}
