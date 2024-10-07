mod builtin;
mod instance;
mod stream;

use std::error;

pub use cursor_icon::CursorIcon;

use bitflags::bitflags;
use downcast_rs::{impl_downcast, Downcast};
pub use instance::{Config, Instance};
pub use stream::{Event, PointerButtons};

use thiserror::Error;

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
    fn backend(&mut self) -> anyhow::Result<crate::backend::Backend>;
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
        tx: crate::shell::channel::Sender<PluginMessage>,
        shell_capabilities: ShellCapabilities,
    ) -> anyhow::Result<()>;
    fn on(&self) -> &str;

    fn handle(&mut self, payload: Vec<u8>) -> anyhow::Result<()>;
}

impl_downcast!(Plugin);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum InstanceError {
    #[error("Running the flutter engine failed")]
    RunFailed,

    #[error("Unable to get the engine proc table")]
    ProcTableFailed,

    #[error("Unable to create aot data")]
    CreateAotData,

    #[error("Aot elf path was not provided")]
    MissingAotPath,
}
