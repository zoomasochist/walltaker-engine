use futures_util::SinkExt;
use serde::{Serialize, Deserialize};
use tokio_tungstenite::tungstenite;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Incoming {
    Welcome,
    Ping { message: u64, },
    ConfirmSubscription { identifier: String },
    #[serde(untagged)]
    Message {
        identifier: String,
        message:    WallpaperUpdate,
    }
}

#[derive(Deserialize)]
pub struct WallpaperUpdate {
    pub id:       usize,
    pub post_url: String,
    pub set_by:   Option<String>,
}

#[derive(Serialize)]
#[serde(tag = "command")]
pub enum Outgoing {
    #[serde(rename = "subscribe")]
    Subscribe { identifier: String, },
    #[serde(rename = "message")]
    Announce {
        data: String,
        identifier: String,
    },
    #[serde(untagged)]
    Check {
        data: String,
        identifier: String,
        command: String,
    },
}

#[derive(Serialize)]
struct Action {
    action: String,
}

#[derive(Serialize)]
struct AnnounceData {
    client: String,
    action: String,
}

#[derive(Serialize)]
pub struct Identifier {
    pub channel: String,
    pub id: usize,
}

fn subscribe_message(id: usize) -> anyhow::Result<String> {
    let inner = Identifier { channel: String::from("LinkChannel"), id };
    let inner = serde_json::to_string(&inner)?;

    let msg = Outgoing::Subscribe { identifier: inner };
    Ok(serde_json::to_string(&msg)?)
}

fn check_message(id: usize) -> anyhow::Result<String> {
    let inner = Identifier { channel: String::from("LinkChannel"), id };
    let inner = serde_json::to_string(&inner)?;

    let data = Action { action: String::from("check"), };
    let data = serde_json::to_string(&data)?;

    let msg = Outgoing::Check {
        data,
        identifier: inner,
        command: String::from("message")
    };

    Ok(serde_json::to_string(&msg)?)
}

fn announce_message(id: usize) -> anyhow::Result<String> {
    let inner = Identifier { channel: String::from("LinkChannel"), id };
    let data = AnnounceData {
        client: format!("WalltakerEngine-chewtoy/{VERSION}"),
        action: String::from("announce_client"),
    };
    let msg = Outgoing::Announce {
        identifier: serde_json::to_string(&inner)?,
        data: serde_json::to_string(&data)?,
    };

    Ok(serde_json::to_string(&msg)?)
}

pub async fn subscribe_to(writer: &crate::Writer, id: usize) -> anyhow::Result<()> {
    let msg = subscribe_message(id)?;
    send(writer, &msg).await?;

    let msg = announce_message(id)?;
    send(writer, &msg).await?;

    Ok(())
}

pub async fn check(writer: &crate::Writer, id: usize) -> anyhow::Result<()> {
    let msg = check_message(id)?;
    send(writer, &msg).await?;

    Ok(())
}

async fn send(to: &crate::Writer, msg: &str) -> anyhow::Result<()> {
    to.lock().await.send(tungstenite::Message::text(msg)).await?;

    Ok(())
}