extern crate reqwest;

use api::Payload;
use config::Config;

pub struct Emitter<'a, 'b> {
    payload: &'a Payload,
    config: &'b Config,
}

enum EmitAction {
    Delete,
    Update,
}

impl<'a, 'b> Emitter<'a, 'b> {
    pub fn new(payload: &'a Payload, config: &'b Config) -> Emitter<'a, 'b> {
        Emitter {
            payload: payload,
            config: config,
        }
    }

    pub fn delete(&self) -> u8 {
        self.emit(EmitAction::Delete)
    }

    pub fn update(&self) -> u8 {
        self.emit(EmitAction::Update)
    }

    fn emit(&self, method: EmitAction) -> u8 {
        let payload_type =
            self.payload.data.get("type").and_then(|type_json| type_json.as_str()).unwrap_or("");

        match self.config.hooks {
                Some(ref all_hooks) => {
                    all_hooks.get(payload_type)
                        .and_then(|hooks| {
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
                }
                _ => None,
            }
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn constructs() {
        unimplemented!()
    }

    #[test]
    fn emits_update() {
        unimplemented!()
    }

    #[test]
    fn update_contains_object() {
        unimplemented!()
    }

    #[test]
    fn emits_delete() {
        unimplemented!()
    }

    #[test]
    fn delete_contains_id() {
        unimplemented!()
    }

    #[test]
    fn calls_all_hooks() {
        unimplemented!()
    }

    #[test]
    fn only_calls_hooks_for_type() {
        unimplemented!()
    }
}
