mod builtin;
mod engine;
mod instance;

use std::sync::{Arc, Mutex};

pub use cursor_icon::CursorIcon;

use downcast_rs::{impl_downcast, Downcast, DowncastSync};
pub use instance::Instance;
use smithay_client_toolkit::reexports::calloop::channel::Sender;
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub enum PointerButtons {
    Primary,
    Secondary,
    Middle,
    Back,
}

use crate::{backend::Surface, shell::sm::PopupParent, Application};

#[derive(Clone)]
pub struct Bundle {
    pub assets: String,
    pub icu_data: String,
    pub aot_elf_path: Option<String>,
}

pub struct ParentInfo {
    pub width: usize,
    pub height: usize,
    pub last_configure: usize,

    pub parent: PopupParent,
}

pub trait Shell: Downcast {
    fn surface(&self) -> Surface;
    fn parent_info(&self) -> ParentInfo;

    fn set_instance(&mut self, instance: Arc<Mutex<Instance>>);
    fn instance(&mut self) -> Arc<Mutex<Instance>>;
}

impl_downcast!(Shell);

pub trait Plugin: Downcast + DowncastSync {
    fn init(
        &mut self,
        shell: Arc<Mutex<dyn Shell>>,
        tx: Sender<EngineRequest>,
    ) -> anyhow::Result<()>;
    fn on(&self) -> &str;

    fn state_changed(&mut self, _state: InstanceState) -> anyhow::Result<()> {
        Ok(())
    }

    fn handle(&mut self, _app: &mut Application<'static>, _data: Vec<u8>) -> anyhow::Result<()> {
        Ok(())
    }
}

impl_downcast!(sync Plugin);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum InstanceError {
    #[error("Aot elf path was not provided")]
    MissingAotPath,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EngineRequest {
    Publish { channel: String, data: Vec<u8> },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum InstanceState {
    Initialized,
    Running,
}
