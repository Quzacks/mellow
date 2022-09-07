use std::{time::Duration};

use futures::{StreamExt, TryStreamExt, SinkExt};
use serde_json::json;
use tokio::{time, sync::mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{json, Error, Result, Client};

use super::{Payload, GatewayOp, GatewayError};

const DISCORD_GATEWAY_URL: &str = "wss://gateway.discord.gg/?v=10&encoding=json";

pub struct DiscordWsClient<'a> {
    pub client: Client<'a>
}

impl<'a> DiscordWsClient<'a> {
    pub async fn open_connection(&self) -> Result<()> {
        let url = url::Url::parse(DISCORD_GATEWAY_URL).unwrap();
        let (t_stream, _) = connect_async(url).await.unwrap();

        let ws_stream = t_stream.map_err(|e| Error::Tungstenite(e));
        let (mut sink, stream) = ws_stream.split();
        
        let (sender, mut receiver) = mpsc::channel(1000);
        
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                sink.send(msg).await.expect("Failed to aggregate message to socket.");
            }
        });
        
        stream.try_for_each(|m| async {
            let payload: Payload = json::from_str(&m.to_string()).unwrap();
            drop(m);

            self.handle_payload(payload, sender.clone()).await
        }).await?;
        
        Ok(())
    }

    async fn handle_payload(&self, payload: Payload, sender: mpsc::Sender<Message>) -> Result<()> {
        println!("{payload:?}");
        let op = GatewayOp::from_code(payload.op);

        if let Some(op) = op {
            use GatewayOp::*;

            match op {
                Hello => {
                    let interval = payload.d.unwrap()["heartbeat_interval"].as_i64().unwrap();
                    let mut interval = time::interval(Duration::from_millis(interval as u64));

                    let c_sender = sender.clone();

                    tokio::spawn(async move {
                        loop {
                            interval.tick().await;
                            send_heartbeat(&c_sender).await;
                        }
                    });

                    auth_client(&self.client, &sender.clone()).await;
                },
                Dispatch => {
                    // TODO: Implementation...
                }
                _ => return Ok(())
            }
        } else {
            return Err(Error::Gateway(GatewayError::InvalidOpCode))
        }

        Ok(())
    }
}

async fn send_heartbeat(sender: &mpsc::Sender<Message>)  {
    let heartbeat = json!({
        "op": GatewayOp::Heartbeat.code(),
        "d": "null"
    });

    sender.send(Message::Text(heartbeat.to_string())).await.unwrap();
}

async fn auth_client<'a>(client: &Client<'a>, sender: &mpsc::Sender<Message>) {
    let auth = json!({
        "op": GatewayOp::Identify.code(),
        "d": {
            "token": client.token,
            "intents": client.intents,
            "properties": {
                "os": "linux",
                "browser": "mellow",
                "device": "mellow"
            }
        }
    });

    sender.send(Message::Text(auth.to_string())).await.expect("Failed to authenticate client.");
}