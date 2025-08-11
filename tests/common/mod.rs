use async_dashscope::Client;

pub fn init_client() -> Client {
    dotenvy::dotenv().unwrap();

    let client = Client::default();

    client
}
