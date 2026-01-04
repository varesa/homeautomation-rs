pub struct HomeAssistant {
    client: reqwest::Client,
    url: String,
    token: String,
}

impl HomeAssistant {
    pub fn new(url: String, token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            token,
        }
    }

    pub async fn service(&self, domain: &str, service: &str, entity_id: &str) {
        println!(
            "Calling service {}/{} with entity_id {}",
            domain, service, entity_id
        );
        let url = format!("{}/api/services/{}/{}", self.url, domain, service);

        let mut body = std::collections::HashMap::new();
        body.insert("entity_id", entity_id);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await;

        match response {
            Ok(res) => {
                if !res.status().is_success() {
                    println!("Failed to call service: {:?}", res.text().await);
                }
            }
            Err(e) => println!("Error sending request to HA: {:?}", e),
        }
    }
}
