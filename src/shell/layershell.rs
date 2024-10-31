use std::{
    cell::RefCell,
    ptr::NonNull,
    rc::Rc,
    sync::{Arc, Mutex},
    usize,
};

use raw_window_handle::{RawWindowHandle, WaylandWindowHandle};
use smithay_client_toolkit::{
    reexports::calloop::channel::Sender,
    shell::{
        wlr_layer::{Anchor, KeyboardInteractivity, Layer, LayerSurface as WlrLayerSurface},
        WaylandSurface,
    },
};
use wayland_client::{protocol::wl_surface::WlSurface, Proxy};

use crate::{
    backend::{Backend, MAIN_SURFACE},
    engine::{self, PointerButtons, Shell},
    shell::app::Request,
};

use super::app::{KeyState, RecreateRequest, RecreateResponse, State};

pub struct LayerSurface {
    backend: Arc<Mutex<Backend>>,
    instance: Option<Rc<RefCell<engine::Instance>>>,
    tx: Sender<Request>,

    id: usize,

    width: usize,
    height: usize,

    surface: Option<WlrLayerSurface>,
}

impl LayerSurface {
    pub fn new(
        id: usize,
        tx: Sender<Request>,
        backend: Arc<Mutex<Backend>>,
        surface: WlrLayerSurface,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            id,
            backend,
            tx,
            surface: Some(surface),
            instance: None,
            width: 0,
            height: 0,
        })
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.surface.as_ref().unwrap().set_anchor(anchor);
        self.surface.as_ref().unwrap().commit();
    }

    pub fn set_exclusive_zone(&mut self, zone: i32) {
        self.surface.as_ref().unwrap().set_exclusive_zone(zone);
    }

    pub fn set_keyboard_interactivity(&mut self, value: KeyboardInteractivity) {
        self.surface
            .as_ref()
            .unwrap()
            .set_keyboard_interactivity(value);
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.surface.as_ref().unwrap().set_layer(layer);
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.surface
            .as_ref()
            .unwrap()
            .set_size(width as u32, height as u32);

        if self.width == 0 && self.height == 0 {
            let ptr =
                NonNull::new(self.surface.as_ref().unwrap().wl_surface().id().as_ptr()
                    as *mut std::ffi::c_void)
                .unwrap();
            let handle = WaylandWindowHandle::new(ptr);
            _ = self
                .backend
                .lock()
                .unwrap()
                .surface(&RawWindowHandle::Wayland(handle), width, height)
                .unwrap();
        } else {
            self.backend
                .lock()
                .unwrap()
                .resize(&MAIN_SURFACE, width, height, 0, 0);
        }

        self.width = width;
        self.height = height;
    }

    pub fn commit(&mut self) {
        self.surface.as_ref().unwrap().commit();
    }
}

impl State for LayerSurface {
    fn id(&self) -> usize {
        self.id.clone()
    }

    fn map(&mut self, response: super::app::RecreateResponse) {
        assert!(self.surface.is_none());

        if let RecreateResponse::Layershell(surface) = response {
            self.surface = Some(surface);
        }

        let ptr = NonNull::new(
            self.surface.as_ref().unwrap().wl_surface().id().as_ptr() as *mut std::ffi::c_void
        )
        .unwrap();
        let handle = WaylandWindowHandle::new(ptr);
        _ = self
            .backend
            .lock()
            .unwrap()
            .surface(&RawWindowHandle::Wayland(handle), self.width, self.height)
            .expect("could create egl surface");
    }

    fn exit(&mut self) {
        if self.instance.is_some() {
            self.instance().borrow_mut().engine_mut().exit().unwrap();
        }
    }

    fn surface(&self) -> &WlSurface {
        self.surface.as_ref().unwrap().wl_surface()
    }

    fn resize(&mut self, width: usize, height: usize, dx: usize, dy: usize) {
        self.width = width;
        self.height = height;

        self.backend
            .lock()
            .unwrap()
            .resize(&MAIN_SURFACE, width, height, dx, dy);
        if self.instance.is_some() {
            self.instance()
                .borrow_mut()
                .engine_mut()
                .resize(width, height)
                .unwrap();
        }
    }

    fn pointer_enter(&mut self, x: f64, y: f64) {
        if self.instance.is_some() {
            self.instance()
                .borrow_mut()
                .engine_mut()
                .pointer_enter(x, y)
                .unwrap();
        }
    }

    fn pointer_leave(&mut self) {
        if self.instance.is_some() {
            self.instance()
                .borrow_mut()
                .engine_mut()
                .pointer_leave()
                .unwrap();
        }
    }

    fn pointer_motion(&mut self, x: f64, y: f64, time: usize) {
        if self.instance.is_some() {
            self.instance()
                .borrow_mut()
                .engine_mut()
                .pointer_motion(x, y, time)
                .unwrap();
        }
    }

    fn pointer_button(&mut self, x: f64, y: f64, time: u32, button: PointerButtons, pressed: bool) {
        if self.instance.is_some() {
            self.instance()
                .borrow_mut()
                .engine_mut()
                .pointer_button(x, y, time, button, pressed)
                .unwrap();
        }
    }

    fn key_event(
        &mut self,
        event: &smithay_client_toolkit::seat::keyboard::KeyEvent,
        state: KeyState,
    ) {
        if self.instance.is_some() {
            if state == KeyState::Pressed {
                self.instance()
                    .borrow_mut()
                    .key_press(event.keysym)
                    .unwrap();
            }
        }
    }
}

impl Shell for LayerSurface {
    fn backend(&mut self) -> anyhow::Result<Arc<Mutex<Backend>>> {
        Ok(self.backend.clone())
    }

    fn hide(&mut self) -> anyhow::Result<()> {
        self.instance().borrow_mut().hide();
        self.pointer_leave();

        self.backend.lock().unwrap().remove(&MAIN_SURFACE);

        let id = self.surface().id();
        self.tx.send(Request::Unmap { id }).unwrap();

        self.surface = None;

        Ok(())
    }

    fn show(&mut self) -> anyhow::Result<()> {
        let id = self.id();
        self.tx
            .send(Request::Recreate {
                id,
                req: RecreateRequest::Layershell,
            })
            .unwrap();

        self.instance().borrow_mut().show();

        Ok(())
    }

    fn set_instance(&mut self, instance: Rc<RefCell<engine::Instance>>) {
        self.instance = Some(instance);
    }

    fn instance(&mut self) -> Rc<RefCell<engine::Instance>> {
        if self.instance.is_none() {
            panic!("instance not set");
        }

        self.instance.as_ref().unwrap().clone()
    }
}
