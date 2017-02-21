use hooks::Payload;
use config::HookConfig;

pub enum EmitAction {
    Delete,
    Update,
}

#[derive(Debug, PartialEq)]
pub struct EmitResponse {
    pub success: Vec<String>,
    pub failure: Vec<String>,
}

impl EmitResponse {
    pub fn results(&self) -> (i64, i64) {
        (self.success.len() as i64, self.failure.len() as i64)
    }
}

pub trait Emitter<'a, 'b> {
    fn new(payload: &'a Payload, config: &'b HookConfig) -> Self;
    fn delete(&self) -> EmitResponse;
    fn update(&self) -> EmitResponse;
}
