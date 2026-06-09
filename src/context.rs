use reqwest::Client as HttpClient;
use serenity::all::{ClientBuilder, Context, prelude::TypeMapKey};

pub struct HttpClientKey;
impl TypeMapKey for HttpClientKey {
    type Value = HttpClient;
}

pub trait SerenityHttpClientExt {
    fn register_http_client(self) -> ClientBuilder;
}

impl SerenityHttpClientExt for ClientBuilder {
    fn register_http_client(self) -> Self {
        self.type_map_insert::<HttpClientKey>(HttpClient::new())
    }
}

pub trait ContextHttpClientExt {
    async fn get_http_client(&self) -> HttpClient;
}

impl ContextHttpClientExt for Context {
    async fn get_http_client(&self) -> HttpClient {
        let data = self.data.read().await;
        data.get::<HttpClientKey>()
            .cloned()
            .expect("HttpClient not registered in context!")
    }
}
