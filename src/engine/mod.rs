mod builtin;
mod engine;
mod instance;

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

pub use cursor_icon::CursorIcon;

use downcast_rs::{impl_downcast, Downcast};
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

use crate::{backend::Backend, Application};

pub struct Bundle {
    pub assets: String,
    pub icu_data: String,
    pub aot_elf_path: Option<String>,
}

pub trait Shell: Downcast {
    fn backend(&mut self) -> anyhow::Result<Arc<Mutex<Backend>>>;

    fn hide(&mut self) -> anyhow::Result<()>;
    fn show(&mut self) -> anyhow::Result<()>;

    fn set_instance(&mut self, instance: Rc<RefCell<Instance>>);
    fn instance(&mut self) -> Rc<RefCell<Instance>>;
}

impl_downcast!(Shell);

pub trait Plugin: Downcast {
    fn init(
        &mut self,
        shell: Rc<RefCell<dyn Shell>>,
        tx: Sender<EngineRequest>,
    ) -> anyhow::Result<()>;
    fn on(&self) -> &str;

    fn handle(&mut self, _app: &mut Application<'static>, _data: Vec<u8>) -> anyhow::Result<()> {
        Ok(())
    }
}

impl_downcast!(Plugin);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum InstanceError {
    #[error("Aot elf path was not provided")]
    MissingAotPath,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EngineRequest {
    Publish { channel: String, data: Vec<u8> },
}
