use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, SubscribeFilter};
use std::time::Duration;

fn entity_name(topic: &str) -> String {
    topic.split('/').nth(3).unwrap().to_string()
}

struct HomeAssistant {
    client: reqwest::Client,
    url: String,
    token: String,
}

impl HomeAssistant {
    fn new(url: String, token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
            token,
        }
    }

    async fn service(&self, domain: &str, service: &str, entity_id: &str) {
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

async fn handle(
    entity: &str,
    state: &str,
    _retain: bool,
    _combine_lights: &mut bool,
    hass: &HomeAssistant,
) {
    println!("Entity: {}, State: {}", entity, state);

    if entity == "valokatkaisijat_etu" || entity == "valokatkaisijat_taka" {
        let service = if state == "on" { "turn_on" } else { "turn_off" };
        hass.service("light", service, "light.z_valot_etu").await;
    }
}

#[tokio::main]
async fn main() {
    let hass_url = std::env::var("HASS_URL").expect("Missing HASS_URL environment variable");
    let hass_token = std::env::var("HASS_TOKEN").expect("Missing HASS_TOKEN environment variable");

    let mut mqttoptions = MqttOptions::new(
        "homeautomation-rs",
        std::env::var("MQTT_HOST").expect("Missing MQTT_HOST environment variable"),
        1883,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    let hass = HomeAssistant::new(hass_url, hass_token);

    let topics = [
        "homeassistant/statestream/event/valokatkaisijat_etu/event_type",
        "homeassistant/statestream/event/valokatkaisijat_taka/event_type",
        "homeassistant/statestream/input_boolean/combine_lights/state",
    ]
    .map(|topic| SubscribeFilter::new(topic.to_string(), QoS::AtMostOnce))
    .to_vec();

    client.try_subscribe_many(topics).unwrap();

    let mut combine_lights = false;

    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                let entity_name = entity_name(&publish.topic);
                let value = String::from_utf8_lossy(&publish.payload).replace('"', "");
                handle(
                    &entity_name,
                    &value,
                    publish.retain,
                    &mut combine_lights,
                    &hass,
                )
                .await;
            }
            Ok(_) => {}
            Err(e) => {
                println!("Error = {:?}", e);
            }
        }
    }
}
