extern crate reqwest;

use api::Payload;
use config::get_config;

pub struct Emitter<'a> {
    payload: &'a Payload,
}

enum EmitAction {
    Delete,
    Update,
}

impl<'a> Emitter<'a> {
    pub fn new(payload: &Payload) -> Emitter {
        Emitter { payload: payload }
    }

    pub fn delete(&self) -> u8 {
        self.emit(EmitAction::Delete)
    }

    pub fn update(&self) -> u8 {
        self.emit(EmitAction::Update)
    }

    fn emit(&self, method: EmitAction) -> u8 {

        get_config()
            .and_then(|config| config.hooks)
            .and_then(|all_hooks| {
                let payload_type =
                    self.payload.data.get("type").and_then(|type_json| type_json.as_str());

                payload_type.and_then(|type_str| {
                    all_hooks.get(type_str).and_then(|hooks| {
                        Some(hooks.iter()
                            .map(|hook| {
                                reqwest::Client::new()
                                    .and_then(|client| {
                                        Ok(match method {
                                            EmitAction::Delete => true,
                                            EmitAction::Update => {
                                                client.post(hook)
                                                    .json(&self.payload)
                                                    .send()
                                                    .is_err()
                                            }
                                        })
                                    })
                                    .unwrap_or(false)

                            })
                            .fold(0, |failures, did_fail| if did_fail {
                                failures + 1
                            } else {
                                failures
                            }))
                    })
                })
            })
            .unwrap_or(0)
    }
}
