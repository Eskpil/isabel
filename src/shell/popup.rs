use raw_window_handle::{RawWindowHandle, WaylandWindowHandle};
use smithay_client_toolkit::{
    reexports::calloop::channel::Sender, shell::xdg::popup::Popup as ToolkitPopup,
};
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::Proxy;

use super::positioner::Positioner;
use super::sm::{PopupParent, RecreateRequest, RecreateResponse, Request};
use super::State;
use crate::engine::{Instance, ParentInfo};
use crate::{
    backend::{Backend, Surface},
    Shell,
};

use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PopupError {
    #[error("popup is already visible")]
    AlreadyVisible,
}

pub struct Popup {
    surface: Surface,
    id: usize,

    tx: Sender<Request>,

    width: usize,
    height: usize,

    instance: Option<Arc<Mutex<Instance>>>,
    inner: Option<ToolkitPopup>,
}

impl Popup {
    pub fn new(
        id: usize,
        tx: Sender<Request>,
        backend: Backend,
        surface: &WlSurface,
    ) -> anyhow::Result<Self> {
        let ptr = NonNull::new(surface.id().as_ptr() as *mut std::ffi::c_void).unwrap();
        let handle = WaylandWindowHandle::new(ptr);

        Ok(Self {
            surface: Surface::new(backend, &RawWindowHandle::Wayland(handle), 1, 1)?,
            id,
            tx,
            width: 1,
            height: 1,
            inner: None,
            instance: None,
        })
    }

    pub fn grab(&self, serial: usize) -> anyhow::Result<()> {
        if let Some(popup) = &self.inner {
            self.tx.send(Request::PopupGrab {
                popup: popup.clone(),
                serial,
            })?;
        }

        Ok(())
    }

    pub fn hide(&mut self) -> anyhow::Result<()> {
        if self.instance.is_some() {
            self.pointer_leave();
            self.instance().lock().unwrap().hide();
        }

        self.surface.clear();

        let id = self.inner.as_ref().unwrap().wl_surface().id();
        self.tx.send(Request::Unmap { id }).unwrap();

        self.inner = None;

        Ok(())
    }

    pub fn show(&mut self, positioner: Positioner, parent: PopupParent) -> anyhow::Result<()> {
        if self.inner.is_some() {
            return Err(PopupError::AlreadyVisible.into());
        }

        let id = self.id();
        self.tx
            .send(Request::Recreate {
                id,
                req: RecreateRequest::Popup(positioner, parent),
            })
            .unwrap();

        if self.instance.is_some() {
            self.instance().lock().unwrap().show();
        }

        Ok(())
    }
}

impl State for Popup {
    fn id(&self) -> usize {
        self.id.clone()
    }

    fn map(&mut self, response: RecreateResponse) {
        if let RecreateResponse::Popup(inner) = response {
            self.inner = Some(inner);
        } else {
            unreachable!()
        }

        let ptr = NonNull::new(
            self.inner.as_ref().unwrap().wl_surface().id().as_ptr() as *mut std::ffi::c_void
        )
        .unwrap();

        let handle = WaylandWindowHandle::new(ptr);

        self.surface
            .recreate(&RawWindowHandle::Wayland(handle), self.width, self.height)
            .expect("could not resize surface");
    }

    fn exit(&mut self) {}

    fn resize(&mut self, width: usize, height: usize, dx: usize, dy: usize) {
        self.width = width;
        self.height = height;

        self.surface
            .resize(width, height, dx, dy)
            .expect("could not resize");

        if self.instance.is_some() {
            println!("resizing popup");

            self.instance()
                .lock()
                .unwrap()
                .engine_mut()
                .resize(width, height)
                .unwrap();
        }
    }

    fn key_event(
        &mut self,
        _event: &smithay_client_toolkit::seat::keyboard::KeyEvent,
        _state: super::sm::KeyState,
    ) {
    }

    fn pointer_enter(&mut self, _x: f64, _y: f64) {}

    fn pointer_leave(&mut self) {}

    fn pointer_button(
        &mut self,
        _x: f64,
        _y: f64,
        _time: u32,
        _button: crate::engine::PointerButtons,
        _pressed: bool,
    ) {
    }

    fn pointer_motion(&mut self, _x: f64, _y: f64, _time: usize) {}
}

impl Shell for Popup {
    fn surface(&self) -> Surface {
        self.surface.clone()
    }

    fn instance(&mut self) -> Arc<Mutex<Instance>> {
        if self.instance.is_none() {
            panic!("instance not set");
        }

        self.instance.as_ref().unwrap().clone()
    }

    fn parent_info(&self) -> crate::engine::ParentInfo {
        let parent = self.inner.as_ref().unwrap().xdg_surface().clone();

        ParentInfo {
            width: self.width,
            height: self.height,
            last_configure: 0,
            parent: PopupParent::XdgSurface(parent),
        }
    }

    fn set_instance(&mut self, instance: Arc<Mutex<crate::Instance>>) {
        self.instance = Some(instance);
    }
}
