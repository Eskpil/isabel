#[derive(Clone, Copy, Debug)]
pub enum PointerButtons {
    Primary,
    Secondary,
    Middle,
    Back,
}

#[derive(Clone, Debug)]
pub enum Event {
    Resized {
        width: u32,
        height: u32,
    },
    Close,

    PointerEnter {
        x: f64,
        y: f64,
    },
    PointerLeave {
        x: f64,
        y: f64,
    },
    PointerMotion {
        x: f64,
        y: f64,
        time: u32,
        serial: u32,
    },
    PointerButton {
        x: f64,
        y: f64,
        button: PointerButtons,
        time: u32,
        pressed: bool,
        serial: u32,
    },
    PointerScroll,

    Keypress {
        symbol: xkeysym::Keysym,
        pressed: bool,
    },
}
