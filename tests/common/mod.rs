use async_dashscope::Client;

pub fn init_client() -> Client {
    let _ = dotenvy::dotenv();

    let client = Client::default();

    client
}
