use homeassistant::HomeAssistant;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS, SubscribeFilter};
use std::collections::HashMap;
use std::time::Duration;

mod homeassistant;

fn entity_name(topic: &str) -> String {
    topic.split('/').nth(3).unwrap().to_string()
}

async fn handle(
    entity: &str,
    state: &str,
    is_retained: bool,
    combine_lights: &mut bool,
    hass: &HomeAssistant,
) {
    println!("Entity: {}, State: {}", entity, state);

    if entity == "combine_lights" {
        *combine_lights = state == "on";
        println!("combine_lights = {}", combine_lights);
        return;
    }

    if state == "arrow_left_click" {
        hass.service("input_boolean", "turn_off", "input_boolean.combine_lights")
            .await;
        return;
    }

    if state == "arrow_right_click" {
        hass.service("input_boolean", "turn_on", "input_boolean.combine_lights")
            .await;
        return;
    }

    let switch_to_light: HashMap<&str, &str> = [
        ("valokatkaisijat_etu", "light.valot_etu"),
        ("valokatkaisijat_taka", "light.valot_taka"),
    ]
    .into();

    if !switch_to_light.contains_key(entity) || is_retained {
        return;
    }

    let targets = if *combine_lights {
        switch_to_light.values().collect()
    } else {
        vec![&switch_to_light[entity]]
    };

    let service = match state {
        "on" | "brightness_up_click" => "turn_on",
        "off" | "brightness_down_click" => "turn_off",
        _ => return,
    };
    for target in targets {
        hass.service("light", service, target).await;
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
