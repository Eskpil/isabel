use crate::shell::channel::Sender;
use anyhow::Error;
use serde::{Deserialize, Serialize};

use crate::engine::{CsdMessage, Plugin, PluginMessage, ShellCapabilities};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SerialEvent {
    serial: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "method", content = "args")]
pub enum Command {
    InitiateMove(SerialEvent),
    Move(SerialEvent),
}

#[derive(Debug, Clone)]
pub struct Decorations {
    tx: Option<Sender<crate::engine::PluginMessage>>,
}

fn publish<T: Serialize>(channel: String, data: &T, tx: &Sender<PluginMessage>) {
    let payload = serde_json::ser::to_vec(data).expect("could not serialize");
    tx.send(PluginMessage::PlatformMessage { channel, payload })
        .unwrap()
}

impl Decorations {
    pub fn new() -> Self {
        Self { tx: None }
    }

    pub fn move_(&mut self, serial: u32) {
        let cmd = Command::Move(SerialEvent { serial });
        publish(self.on().to_lowercase(), &cmd, self.tx.as_ref().unwrap());
    }
}

impl crate::engine::Plugin for Decorations {
    fn init(
        &mut self,
        tx: Sender<crate::engine::PluginMessage>,
        shell_capabilities: crate::engine::ShellCapabilities,
    ) -> anyhow::Result<()> {
        self.tx = Some(tx);

        if !shell_capabilities.contains(ShellCapabilities::CLIENT_SIDE_DECORATIONS) {
            return Err(Error::msg("shell does not support client side decorations"));
        }

        Ok(())
    }

    fn on(&self) -> &str {
        "isabel/decorations"
    }

    fn handle(&mut self, payload: Vec<u8>) -> anyhow::Result<()> {
        let command: Command = serde_json::from_slice(&payload[..])?;

        match command {
            Command::InitiateMove(e) => {
                let msg = PluginMessage::Csd(CsdMessage::Move(e.serial));
                self.tx.as_mut().unwrap().send(msg)?;
            }
            Command::Move(_) => unreachable!("should only be sent by instance"),
        }

        Ok(())
    }
}
