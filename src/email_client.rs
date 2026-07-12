use reqwest::Client;

#[derive(Clone)]
pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: String,
}

impl EmailClient {
    pub fn new(base_url: &str) -> Self {
        Self { http_client: Client::new(), base_url: base_url.to_string() }
    }

    pub async fn send_email(&self, path: i32) -> Result<(), reqwest::Error> {
        let url = format!("{}/api/v2/item-fling-effect/{}", self.base_url, path);
        let response =
            self.http_client.get(&url).header("Content-Type", "application/json").send().await?;

        let body = response.text().await?;
        println!("Body: {}", body);

        Ok(())
    }
}
