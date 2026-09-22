use async_dashscope::Client;
use async_dashscope::config::{
    ASYNC_HEADER, ConfigBuilder, OSS_RESOURCE_RESOLVE_HEADER, WORKSPACE_HEADER,
};
use async_dashscope::operation::generation::{GenerationParamBuilder, InputBuilder, MessageBuilder};

#[test]
fn workspace_resolves_maas_placeholder() {
    let config = ConfigBuilder::default()
        .api_key("sk-test")
        .workspace("ws_abc")
        .api_base("https://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api/v1")
        .build()
        .unwrap();

    assert_eq!(
        config
            .try_url("/services/aigc/text-generation/generation")
            .unwrap(),
        "https://ws_abc.ap-southeast-1.maas.aliyuncs.com/api/v1/services/aigc/text-generation/generation"
    );
    assert_eq!(config.headers().get(WORKSPACE_HEADER).unwrap(), "ws_abc");
}

#[test]
fn invalid_workspace_is_rejected() {
    let config = ConfigBuilder::default()
        .api_key("sk-test")
        .workspace("bad id")
        .api_base("https://{workspace_id}.ap-southeast-1.maas.aliyuncs.com/api/v1")
        .build()
        .unwrap();

    assert!(config.try_url("/test").is_err());
}

#[test]
fn header_scopes() {
    let config = ConfigBuilder::default().api_key("sk-test").build().unwrap();

    assert!(config.headers().get(OSS_RESOURCE_RESOLVE_HEADER).is_none());
    assert!(config.headers().get(ASYNC_HEADER).is_none());
    assert_eq!(
        config
            .oss_headers()
            .get(OSS_RESOURCE_RESOLVE_HEADER)
            .unwrap(),
        "enable"
    );
    assert_eq!(config.async_headers().get(ASYNC_HEADER).unwrap(), "enable");
}

#[test]
fn client_with_workspace() {
    let client = Client::new()
        .with_api_key("sk-test".to_string())
        .with_workspace("ws_client".to_string());

    assert_eq!(client.config().workspace(), Some("ws_client"));
    assert_eq!(
        client.config().headers().get(WORKSPACE_HEADER).unwrap(),
        "ws_client"
    );
}

#[test]
fn websocket_url_resolution() {
    let config = ConfigBuilder::default()
        .api_key("sk-test")
        .workspace("ws_ws")
        .websocket_base(
            "wss://{workspace_id}.cn-hongkong.maas.aliyuncs.com/api-ws/v1/inference",
        )
        .build()
        .unwrap();

    assert_eq!(
        config.try_websocket_url().unwrap(),
        "wss://ws_ws.cn-hongkong.maas.aliyuncs.com/api-ws/v1/inference"
    );
}

#[test]
fn plugins_not_serialized_in_request_body() {
    let request = GenerationParamBuilder::default()
        .model("qwen-plus")
        .input(
            InputBuilder::default()
                .messages(vec![
                    MessageBuilder::default()
                        .user()
                        .content("hi")
                        .build()
                        .unwrap(),
                ])
                .build()
                .unwrap(),
        )
        .plugins(serde_json::json!({"search": {"enable": true}}))
        .build()
        .unwrap();

    let value = serde_json::to_value(&request).unwrap();
    assert!(value.get("plugins").is_none());
}
