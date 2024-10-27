mod builtin;
mod engine;
mod instance;

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub use cursor_icon::CursorIcon;

use bitflags::bitflags;
use downcast_rs::{impl_downcast, Downcast};
pub use instance::Instance;
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub enum PointerButtons {
    Primary,
    Secondary,
    Middle,
    Back,
}

use crate::backend::Backend;

pub struct Bundle {
    pub assets: String,
    pub icu_data: String,
    pub aot_elf_path: Option<String>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ShellCapabilities: u32 {
        const TITLE = 0x001;
        const POPUP = 0x002;
        const MOUSE_CURSOR = 0x003;
        const CLIENT_SIDE_DECORATIONS = 0x004;
    }
}

pub trait Shell: Downcast {
    fn backend(&mut self) -> anyhow::Result<Arc<Mutex<Backend>>>;
    fn set_cursor_icon(&mut self, cursor_icon: cursor_icon::CursorIcon) -> anyhow::Result<()>;
    fn capabilities(&self) -> ShellCapabilities;

    fn set_instance(&mut self, instance: Instance);
    fn instance_mut(&mut self) -> &mut Instance;
}

impl_downcast!(Shell);

pub trait Plugin: Downcast {
    fn init(&mut self, shell: Rc<RefCell<dyn Shell>>) -> anyhow::Result<()>;
    fn on(&self) -> &str;

    fn handle(&mut self, payload: Vec<u8>) -> anyhow::Result<()>;
}

impl_downcast!(Plugin);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum InstanceError {
    #[error("Aot elf path was not provided")]
    MissingAotPath,
}
