mod builtin;
mod instance;
mod stream;

pub use cursor_icon::CursorIcon;

use bitflags::bitflags;
use downcast_rs::{impl_downcast, Downcast};
pub use instance::Instance;
pub use stream::{Event, PointerButtons};

use calloop::channel::Sender;

use backend::Backend;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ShellCapabilities: u32 {
        const TITLE = 0x001;
        const POPUP = 0x002;
        const MOUSE_CURSOR = 0x003;
        const CLIENT_SIDE_DECORATIONS = 0x004;
    }
}

pub trait Shell {
    fn backend(&mut self) -> anyhow::Result<Backend>;
    fn exited(&self) -> bool;

    fn set_cursor_icon(&mut self, cursor_icon: cursor_icon::CursorIcon) -> anyhow::Result<()>;

    fn capabilities(&self) -> ShellCapabilities;

    fn initiate_move(&mut self, serial: u32);

    fn set_instance(&mut self, instance: Instance);
    fn instance_mut(&mut self) -> &mut Instance;
}

#[derive(Debug)]
pub enum CsdMessage {
    Move(u32),
}

#[derive(Debug)]
pub enum PluginMessage {
    Csd(CsdMessage),
    SetCursor { icon: CursorIcon },
    PlatformMessage { payload: Vec<u8>, channel: String },
}

pub trait Plugin: Downcast {
    fn init(
        &mut self,
        tx: Sender<PluginMessage>,
        shell_capabilities: ShellCapabilities,
    ) -> anyhow::Result<()>;
    fn on(&self) -> &str;

    fn handle(&mut self, payload: Vec<u8>) -> anyhow::Result<()>;
}

impl_downcast!(Plugin);
