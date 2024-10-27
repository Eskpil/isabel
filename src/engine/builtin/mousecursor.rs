use std::{cell::RefCell, rc::Rc};

use crate::Shell;
use cursor_icon::CursorIcon;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MouseCursor {
    ActivateSystemCursor { device: i32, kind: MouseCursorKind },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MouseCursorKind {
    None,
    Basic,
    Click,
    Forbidden,
    Wait,
    Progress,
    ContextMenu,
    Help,
    Text,
    VerticalText,
    Cell,
    Precise,
    Move,
    Grab,
    Grabbing,
    NoDrop,
    Alias,
    Copy,
    Disappearing,
    AllScroll,
    ResizeLeftRight,
    ResizeUpDown,
    ResizeUpLeftDownRight,
    ResizeUpRightDownLeft,
    ResizeUp,
    ResizeDown,
    ResizeLeft,
    ResizeRight,
    ResizeUpLeft,
    ResizeUpRight,
    ResizeDownLeft,
    ResizeDownRight,
    ResizeColumn,
    ResizeRow,
    ZoomIn,
    ZoomOut,
}

#[derive(Clone)]
pub struct Mousecursor {
    shell: Option<Rc<RefCell<dyn Shell>>>,
}

impl Mousecursor {
    pub fn new() -> Self {
        Self { shell: None }
    }
}

impl crate::engine::Plugin for Mousecursor {
    fn init(&mut self, shell: Rc<RefCell<dyn Shell>>) -> anyhow::Result<()> {
        self.shell = Some(shell);
        Ok(())
    }

    fn on(&self) -> &str {
        "flutter/mousecursor"
    }

    fn handle(&mut self, payload: Vec<u8>) -> anyhow::Result<()> {
        let message: MouseCursor = crate::codec::from_slice(&payload[..])?;

        match message {
            MouseCursor::ActivateSystemCursor { kind, .. } => {
                let icon = match kind {
                    MouseCursorKind::None => unreachable!("pointer kind None"),
                    MouseCursorKind::Basic => CursorIcon::Default,
                    MouseCursorKind::Click => CursorIcon::Pointer,
                    MouseCursorKind::Forbidden => CursorIcon::NotAllowed,
                    MouseCursorKind::Wait => CursorIcon::Wait,
                    MouseCursorKind::Progress => CursorIcon::Progress,
                    MouseCursorKind::ContextMenu => CursorIcon::ContextMenu,
                    MouseCursorKind::Help => CursorIcon::Help,
                    MouseCursorKind::Text => CursorIcon::Text,
                    MouseCursorKind::VerticalText => CursorIcon::VerticalText,
                    MouseCursorKind::Cell => CursorIcon::Cell,
                    MouseCursorKind::Precise => CursorIcon::Crosshair,
                    MouseCursorKind::Move => CursorIcon::Move,
                    MouseCursorKind::Grab => CursorIcon::Grab,
                    MouseCursorKind::Grabbing => CursorIcon::Grabbing,
                    MouseCursorKind::NoDrop => CursorIcon::NoDrop,
                    MouseCursorKind::Alias => CursorIcon::Alias,
                    MouseCursorKind::Copy => CursorIcon::Copy,
                    MouseCursorKind::Disappearing => unreachable!("unnsuported on linux"),
                    MouseCursorKind::AllScroll => CursorIcon::AllScroll,
                    MouseCursorKind::ResizeLeftRight => CursorIcon::EwResize,
                    MouseCursorKind::ResizeUpDown => CursorIcon::NsResize,
                    MouseCursorKind::ResizeUpLeftDownRight => CursorIcon::NwseResize,
                    MouseCursorKind::ResizeUpRightDownLeft => CursorIcon::NeswResize,
                    MouseCursorKind::ResizeUp => CursorIcon::NResize,
                    MouseCursorKind::ResizeDown => CursorIcon::SResize,
                    MouseCursorKind::ResizeLeft => CursorIcon::EResize,
                    MouseCursorKind::ResizeRight => CursorIcon::WResize,
                    MouseCursorKind::ResizeUpLeft => CursorIcon::NeResize,
                    MouseCursorKind::ResizeUpRight => CursorIcon::NwResize,
                    MouseCursorKind::ResizeDownLeft => CursorIcon::SeResize,
                    MouseCursorKind::ResizeDownRight => CursorIcon::SwResize,
                    MouseCursorKind::ResizeColumn => CursorIcon::ColResize,
                    MouseCursorKind::ResizeRow => CursorIcon::RowResize,
                    MouseCursorKind::ZoomIn => CursorIcon::ZoomIn,
                    MouseCursorKind::ZoomOut => CursorIcon::ZoomOut,
                };

                self.shell
                    .clone()
                    .unwrap()
                    .borrow_mut()
                    .set_cursor_icon(icon)?;
            }
        }

        Ok(())
    }
}
