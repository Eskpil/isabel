use bitflags::bitflags;
use isabel_rs::{positioner::Positioner, shell::sm::PopupParent, Gravity, PopupAnchor};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ModuleCapabilities: u32 {
        const POPOP = 0x0001;
        const WINDOW = 0x002;
    }
}

pub struct PopupInfo {
    pub anchor: PopupAnchor,
    pub gravity: Gravity,

    pub width: usize,
    pub height: usize,
}

pub enum ShowRequest {
    Popup(Positioner, PopupParent),
}

pub trait Module {
    fn name(&self) -> &str;
    fn run(&self);

    fn capabilities(&self) -> ModuleCapabilities;
    fn popup_info(&self) -> Option<PopupInfo> {
        None
    }

    fn show(&self, req: ShowRequest) -> anyhow::Result<()>;
    fn hide(&self) -> anyhow::Result<()>;
}
