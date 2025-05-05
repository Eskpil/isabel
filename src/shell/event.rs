use crate::engine::PointerButtons;

use super::sm::KeyState;

#[derive(Clone, Copy)]
pub enum Event {
    Key {
        state: KeyState,
        symbol: xkeysym::Keysym,
        time: u64,
    },
    PointerEnter {
        x: f64,
        y: f64,
    },
    PointerLeave {},
    PointerMotion {
        x: f64,
        y: f64,
        time: u64,
    },
    PointerButton {
        x: f64,
        y: f64,
        time: u64,
        button: PointerButtons,
        state: KeyState,
    },
    Resize {
        width: usize,
        height: usize,
        scale: f64,
    },
    PointerAxis {
        horizontal: f64,
        vertical: f64,
        time: u64,
    },

    Exit,
    Hide,
    Show,
}
