use serde::{Deserialize, Serialize};
use smithay_client_toolkit::reexports::calloop::channel::Sender;

use crate::{
    engine::{EngineRequest, Plugin},
    sm::KeyState,
    Shell,
};

#[derive(Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Esc,
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub struct KeyData {
    pub key: Key,
    pub state: KeyState,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(tag = "method", content = "args")]
pub enum KeyEvent {
    #[serde(rename = "Keys.event")]
    Event(KeyData),
}

#[derive(Clone)]
pub struct Keys {
    tx: Option<Sender<EngineRequest>>,
}

impl Keys {
    pub fn new() -> Self {
        Self { tx: None }
    }

    fn translate_from_xkeysym(symbol: xkeysym::Keysym) -> Option<Key> {
        match symbol {
            xkeysym::Keysym::Escape => Some(Key::Esc),
            xkeysym::Keysym::Up => Some(Key::ArrowUp),
            xkeysym::Keysym::Down => Some(Key::ArrowDown),
            xkeysym::Keysym::Right => Some(Key::ArrowRight),
            xkeysym::Keysym::Left => Some(Key::ArrowLeft),
            _ => None,
        }
    }

    pub fn send_key(&mut self, symbol: xkeysym::Keysym, state: KeyState) {
        if let Some(key) = Self::translate_from_xkeysym(symbol) {
            let data = KeyData { key, state };
            self.publish(&KeyEvent::Event(data));
        }
    }

    fn publish(&mut self, event: &KeyEvent) {
        // Serialize to a JSON Value first
        let json_value = serde_json::to_value(vec![event]).expect("Failed to serialize key event");

        self.tx
            .as_ref()
            .unwrap()
            .send(EngineRequest::Publish {
                channel: self.on().to_owned(),
                data: json_value.to_string().into_bytes(),
            })
            .unwrap();
    }
}

impl crate::engine::Plugin for Keys {
    fn init(
        &mut self,
        _shell: &mut Box<dyn Shell>,
        tx: Sender<EngineRequest>,
    ) -> anyhow::Result<()> {
        self.tx = Some(tx);
        Ok(())
    }

    fn on(&self) -> &str {
        "isabel/keys"
    }
}
