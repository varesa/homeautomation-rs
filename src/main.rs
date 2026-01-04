use rumqttc::{AsyncClient, ConnectionError, Event, MqttOptions, Packet, QoS, SubscribeFilter};
use std::time::Duration;

fn entity_name(topic: &str) -> String {
    topic.split('/').nth(3).unwrap().to_string()
}

fn handle(entity: &str, state: &str, retain: bool, combine_lights: &mut bool) {
    println!("Entity: {}, State: {}, Retain: {}", entity, state, retain);
}

#[tokio::main]
async fn main() {
    let mut mqttoptions = MqttOptions::new("homeautomation-rs", "mqtt.apps.okd.p4.esav.fi", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

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
                handle(&entity_name, &value, publish.retain, &mut combine_lights);
            }
            Ok(_) => {}
            Err(e) => {
                println!("Error = {:?}", e);
            }
        }
    }
}
