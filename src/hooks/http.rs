extern crate reqwest;
extern crate serde_json;

use serde_json::Value as Json;
use self::reqwest::header::{Authorization, Basic, UserAgent};
use self::reqwest::{Method, StatusCode};

use std::collections::BTreeMap;

use hooks::{EmitAction, EmitResponse, Emitter, Payload};
use config::HookConfig;

#[derive(Debug, PartialEq)]
pub struct HttpEmitter<'a, 'b> {
    payload: &'a Payload,
    config: &'b HookConfig,
}

impl<'a, 'b> HttpEmitter<'a, 'b> {
    fn payload_type(&self) -> &str {
        self.payload
            .data
            .get("type")
            .and_then(|type_json| type_json.as_str())
            .unwrap_or("")
    }

    fn hooks(&self) -> Option<&Vec<BTreeMap<String, String>>> {
        self.config.get(self.payload_type())
    }

    fn emit(&self, method: EmitAction) -> EmitResponse {
        let hook_results = self.hooks()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|hook| {
                hook.get("url").map(|base_url| {
                    let user = hook.get("username")
                        .map(|user_ref| user_ref.to_owned())
                        .unwrap_or("".to_string());
                    let pass = hook.get("password").map(|pass_ref| pass_ref.to_owned());

                    let mut url = base_url.clone();

                    if method == EmitAction::Delete {
                        if let &Json::String(ref id) = &self.payload.data["id"] {
                            url.push_str(id);
                            url.push('/');
                        }
                    }

                    (url, user, pass)
                })
            })
            .map(|(url, user, pass)| {
                let client = reqwest::Client::new();

                let mut req = match method {
                    EmitAction::Delete => client.request(Method::Delete, url.as_str()),
                    EmitAction::Update => client.post(url.as_str()),
                };

                req.header(Authorization(Basic {
                    username: user,
                    password: pass,
                })).header(UserAgent::new("MM-API-NOTIFY"))
                    .json(&self.payload);

                let status = req.send().ok().map_or_else(
                    || false,
                    |resp| match resp.status() {
                        StatusCode::Ok => true,
                        _ => false,
                    },
                );

                (url, status)
            })
            .fold((vec![], vec![]), |(mut pass, mut fail), (hook, status)| {
                if status == true {
                    pass.push(hook.to_owned())
                } else {
                    fail.push(hook.to_owned())
                }

                (pass, fail)
            });

        EmitResponse {
            success: hook_results.0,
            failure: hook_results.1,
        }
    }
}

impl<'a, 'b> Emitter<'a, 'b> for HttpEmitter<'a, 'b> {
    fn new(payload: &'a Payload, config: &'b HookConfig) -> HttpEmitter<'a, 'b> {
        HttpEmitter {
            payload: payload,
            config: config,
        }
    }

    fn delete(&self) -> EmitResponse {
        self.emit(EmitAction::Delete)
    }

    fn update(&self) -> EmitResponse {
        self.emit(EmitAction::Update)
    }
}

#[cfg(test)]
mod tests {

    use mockito;
    use mockito::mock;
    use serde_json::Value as Json;

    use std::collections::BTreeMap;

    use hooks::{EmitResponse, Emitter, HttpEmitter, Payload};

    #[test]
    fn emits_update() {
        let test_response = String::from("{\"name\":\"value\"}");

        let _m = mock("POST", "/http_emit_update_test/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test_response.as_str())
            .create();

        let mut endpoint = mockito::SERVER_URL.to_string();
        endpoint.push_str("/http_emit_update_test/");

        let mut hook = BTreeMap::new();
        hook.insert("url".to_string(), endpoint.to_string());

        let mut config = BTreeMap::new();
        config.insert("show".to_string(), vec![hook]);

        if let Json::Object(payload_map) = json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        }) {
            let payload = Payload { data: payload_map };
            let emit = HttpEmitter::new(&payload, &config);

            let emit_resp = EmitResponse {
                success: vec![endpoint.to_string()],
                failure: vec![],
            };

            assert_eq!(emit.update(), emit_resp)
        } else {
            panic!("Failed to create payload map")
        }
    }

    #[test]
    fn update_contains_object() {
        let payload_data = json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        });

        let req_data = json!({ "data": payload_data });

        let _m = mock("POST", "/http_update_contains_object_test/")
            .with_status(200)
            .match_body(req_data.to_string().as_str())
            .create();

        let mut endpoint = mockito::SERVER_URL.to_string();
        endpoint.push_str("/http_update_contains_object_test/");

        let mut hook = BTreeMap::new();
        hook.insert("url".to_string(), endpoint.to_string());

        let mut config = BTreeMap::new();
        config.insert("show".to_string(), vec![hook]);

        let payload_map = payload_data.as_object().unwrap();
        let payload = Payload {
            data: payload_map.to_owned(),
        };
        let emit = HttpEmitter::new(&payload, &config);

        let emit_resp = EmitResponse {
            success: vec![endpoint.to_string()],
            failure: vec![],
        };

        assert_eq!(emit.update(), emit_resp);
    }

    #[test]
    fn emits_delete() {
        let _m = mock("DELETE", "/http_emit_delete_test/test-child/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .create();

        let mut endpoint = mockito::SERVER_URL.to_string();
        endpoint.push_str("/http_emit_delete_test/");

        let mut hook = BTreeMap::new();
        hook.insert("url".to_string(), endpoint.to_string());

        let mut config = BTreeMap::new();
        config.insert("show".to_string(), vec![hook]);

        if let Json::Object(payload_map) = json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        }) {
            let payload = Payload { data: payload_map };
            let emit = HttpEmitter::new(&payload, &config);

            let mut delete_endpoint = endpoint.clone();
            delete_endpoint.push_str("test-child/");

            let emit_resp = EmitResponse {
                success: vec![delete_endpoint],
                failure: vec![],
            };

            assert_eq!(emit.delete(), emit_resp)
        } else {
            panic!("Failed to create payload map")
        }
    }

    #[test]
    fn calls_all_hooks_for_type() {
        let test_response = String::from("{\"name\":\"value\"}");

        let _m = mock("POST", "/http_calls_all_hooks_for_type_test/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test_response.as_str())
            .create();

        let _m = mock("POST", "/http_calls_all_hooks_for_type_test_2/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test_response.as_str())
            .create();

        let mut endpoint1 = mockito::SERVER_URL.to_string();
        endpoint1.push_str("/http_calls_all_hooks_for_type_test/");

        let mut endpoint2 = mockito::SERVER_URL.to_string();
        endpoint2.push_str("/http_calls_all_hooks_for_type_test_2/");

        let mut hook1 = BTreeMap::new();
        hook1.insert("url".to_string(), endpoint1.to_string());

        let mut hook2 = BTreeMap::new();
        hook2.insert("url".to_string(), endpoint2.to_string());

        let mut config = BTreeMap::new();
        config.insert("show".to_string(), vec![hook1, hook2]);

        if let Json::Object(payload_map) = json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        }) {
            let payload = Payload { data: payload_map };
            let emit = HttpEmitter::new(&payload, &config);

            let emit_resp = EmitResponse {
                success: vec![endpoint1.to_string(), endpoint2.to_string()],
                failure: vec![],
            };

            assert_eq!(emit.update(), emit_resp)
        } else {
            panic!("Failed to create payload map")
        }
    }

    #[test]
    fn only_calls_hooks_for_type() {
        let test_response = String::from("{\"name\":\"value\"}");

        let _m = mock("POST", "/http_only_calls_hooks_for_type_test/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test_response.as_str())
            .create();

        let mut endpoint1 = mockito::SERVER_URL.to_string();
        endpoint1.push_str("/http_only_calls_hooks_for_type_test/");

        let mut hook1 = BTreeMap::new();
        hook1.insert("url".to_string(), endpoint1.to_string());

        let mut endpoint2 = mockito::SERVER_URL.to_string();
        endpoint2.push_str("/http_only_calls_hooks_for_type_test_2/");

        let mut hook2 = BTreeMap::new();
        hook2.insert("url".to_string(), endpoint2.to_string());

        let mut config = BTreeMap::new();
        config.insert("show".to_string(), vec![hook1]);
        config.insert("asset".to_string(), vec![hook2]);

        if let Json::Object(payload_map) = json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        }) {
            let payload = Payload { data: payload_map };
            let emit = HttpEmitter::new(&payload, &config);

            let emit_resp = EmitResponse {
                success: vec![endpoint1.to_string()],
                failure: vec![],
            };

            assert_eq!(emit.update(), emit_resp)
        } else {
            panic!("Failed to create payload map")
        }
    }

    #[test]
    fn skips_hooks_without_url() {
        let test_response = String::from("{\"name\":\"value\"}");

        let _m = mock("POST", "/http_skips_hooks_without_url_test/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test_response.as_str())
            .create();

        let hook = BTreeMap::new();

        let mut config = BTreeMap::new();
        config.insert("show".to_string(), vec![hook]);

        if let Json::Object(payload_map) = json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        }) {
            let payload = Payload { data: payload_map };
            let emit = HttpEmitter::new(&payload, &config);

            let emit_resp = EmitResponse {
                success: vec![],
                failure: vec![],
            };

            assert_eq!(emit.update(), emit_resp)
        } else {
            panic!("Failed to create payload map")
        }
    }

    #[test]
    fn handles_hooks_without_auth() {
        let test_response = String::from("{\"name\":\"value\"}");

        let _m = mock("POST", "/http_handles_hooks_without_auth_test/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test_response.as_str())
            .create();

        let mut endpoint = mockito::SERVER_URL.to_string();
        endpoint.push_str("/http_handles_hooks_without_auth_test/");

        let mut hook = BTreeMap::new();
        hook.insert("url".to_string(), endpoint.to_string());

        let mut config = BTreeMap::new();
        config.insert("show".to_string(), vec![hook]);

        if let Json::Object(payload_map) = json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        }) {
            let payload = Payload { data: payload_map };
            let emit = HttpEmitter::new(&payload, &config);

            let emit_resp = EmitResponse {
                success: vec![endpoint.to_string()],
                failure: vec![],
            };

            assert_eq!(emit.update(), emit_resp)
        } else {
            panic!("Failed to create payload map")
        }
    }

    #[test]
    fn handles_hooks_with_auth() {
        let test_response = String::from("{\"name\":\"value\"}");

        let _m = mock("POST", "/http_handles_hooks_with_auth_test/")
            .match_header("Authorization", "Basic aGVsbG86d29ybGQ=")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test_response.as_str())
            .create();

        let mut endpoint = mockito::SERVER_URL.to_string();
        endpoint.push_str("/http_handles_hooks_with_auth_test/");

        let mut hook = BTreeMap::new();
        hook.insert("url".to_string(), endpoint.to_string());
        hook.insert("username".to_string(), "hello".to_string());
        hook.insert("password".to_string(), "world".to_string());

        let mut config = BTreeMap::new();
        config.insert("show".to_string(), vec![hook]);

        if let Json::Object(payload_map) = json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        }) {
            let payload = Payload { data: payload_map };
            let emit = HttpEmitter::new(&payload, &config);

            let emit_resp = EmitResponse {
                success: vec![endpoint.to_string()],
                failure: vec![],
            };

            assert_eq!(emit.update(), emit_resp)
        } else {
            panic!("Failed to create payload map")
        }
    }

    #[test]
    fn emits_json_content_type_header() {
        let test_response = String::from("{\"name\":\"value\"}");

        let _m = mock("POST", "/http_emits_json_content_type_header_test/")
            .match_header("content-type", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(test_response.as_str())
            .create();

        let mut endpoint = mockito::SERVER_URL.to_string();
        endpoint.push_str("/http_emits_json_content_type_header_test/");

        let mut hook = BTreeMap::new();
        hook.insert("url".to_string(), endpoint.to_string());

        let mut config = BTreeMap::new();
        config.insert("show".to_string(), vec![hook]);

        if let Json::Object(payload_map) = json!({
            "id": "test-child",
            "type": "show",
            "updated_at": "2017-01-01T00:00:00Z",
            "parent": {
                "id": "test-parent",
                "parent": null,
                "updated_at": "2017-01-01T00:00:00Z",
                "type": "franchise"
            }
        }) {
            let payload = Payload { data: payload_map };
            let emit = HttpEmitter::new(&payload, &config);

            let emit_resp = EmitResponse {
                success: vec![endpoint.to_string()],
                failure: vec![],
            };

            assert_eq!(emit.update(), emit_resp)
        } else {
            panic!("Failed to create payload map")
        }
    }
}
