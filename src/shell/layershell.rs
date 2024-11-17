use std::{
    ptr::NonNull,
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
use wayland_client::Proxy;

use crate::{
    backend::{Backend, Surface},
    engine::{self, PointerButtons, Shell},
};

use super::sm::{KeyState, PopupParent, RecreateRequest, RecreateResponse, Request, State};

pub struct LayerSurface {
    backend: Backend,
    surface: Surface,
    instance: Option<Arc<Mutex<engine::Instance>>>,
    tx: Sender<Request>,

    id: usize,

    width: usize,
    height: usize,

    inner: Option<WlrLayerSurface>,
}

impl LayerSurface {
    pub fn new(
        id: usize,
        tx: Sender<Request>,
        backend: Backend,
        inner: WlrLayerSurface,
    ) -> anyhow::Result<Self> {
        let ptr = NonNull::new(inner.wl_surface().id().as_ptr() as *mut std::ffi::c_void).unwrap();
        let handle = WaylandWindowHandle::new(ptr);
        let surface = Surface::new(backend, &RawWindowHandle::Wayland(handle), 1, 1)?;

        Ok(Self {
            id,
            backend,
            surface,
            tx,
            inner: Some(inner),
            instance: None,
            width: 0,
            height: 0,
        })
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.inner.as_ref().unwrap().set_anchor(anchor);
        self.inner.as_ref().unwrap().commit();
    }

    pub fn set_exclusive_zone(&mut self, zone: i32) {
        self.inner.as_ref().unwrap().set_exclusive_zone(zone);
    }

    pub fn set_keyboard_interactivity(&mut self, value: KeyboardInteractivity) {
        self.inner
            .as_ref()
            .unwrap()
            .set_keyboard_interactivity(value);
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.inner.as_ref().unwrap().set_layer(layer);
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.inner
            .as_ref()
            .unwrap()
            .set_size(width as u32, height as u32);

        self.surface
            .resize(width, height, 0, 0)
            .expect("could not resize layer shell surface");

        self.width = width;
        self.height = height;
    }

    pub fn commit(&mut self) {
        self.inner.as_ref().unwrap().commit();
    }

    fn hide(&mut self) -> anyhow::Result<()> {
        self.instance().lock().unwrap().hide();
        self.pointer_leave();

        self.surface.clear();

        let id = self.inner.as_ref().unwrap().wl_surface().id();
        self.tx.send(Request::Unmap { id }).unwrap();

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

        self.instance().lock().unwrap().show();

        Ok(())
    }
}

impl State for LayerSurface {
    fn id(&self) -> usize {
        self.id.clone()
    }

    fn map(&mut self, response: super::sm::RecreateResponse) {
        assert!(self.inner.is_none());

        if let RecreateResponse::Layershell(inner) = response {
            self.inner = Some(inner);
        }

        let ptr = NonNull::new(
            self.inner.as_ref().unwrap().wl_surface().id().as_ptr() as *mut std::ffi::c_void
        )
        .unwrap();
        let handle = WaylandWindowHandle::new(ptr);
        self.surface
            .recreate(&RawWindowHandle::Wayland(handle), self.width, self.height)
            .expect("could not recreate surface");
    }

    fn exit(&mut self) {
        if self.instance.is_some() {
            self.instance().lock().unwrap().engine_mut().exit().unwrap();
        }
    }

    fn resize(&mut self, width: usize, height: usize, dx: usize, dy: usize) {
        self.width = width;
        self.height = height;

        self.surface
            .resize(width, height, dx, dy)
            .expect("could not resize");

        if self.instance.is_some() {
            self.instance()
                .lock()
                .unwrap()
                .engine_mut()
                .resize(width, height)
                .unwrap();
        }
    }

    fn pointer_enter(&mut self, x: f64, y: f64) {
        if self.instance.is_some() {
            self.instance()
                .lock()
                .unwrap()
                .engine_mut()
                .pointer_enter(x, y)
                .unwrap();
        }
    }

    fn pointer_leave(&mut self) {
        if self.instance.is_some() {
            self.instance()
                .lock()
                .unwrap()
                .engine_mut()
                .pointer_leave()
                .unwrap();
        }
    }

    fn pointer_motion(&mut self, x: f64, y: f64, time: usize) {
        if self.instance.is_some() {
            self.instance()
                .lock()
                .unwrap()
                .engine_mut()
                .pointer_motion(x, y, time)
                .unwrap();
        }
    }

    fn pointer_button(&mut self, x: f64, y: f64, time: u32, button: PointerButtons, pressed: bool) {
        if self.instance.is_some() {
            self.instance()
                .lock()
                .unwrap()
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
                    .lock()
                    .unwrap()
                    .key_press(event.keysym)
                    .unwrap();
            }
        }
    }
}

impl Shell for LayerSurface {
    fn surface(&self) -> Surface {
        self.surface.clone()
    }

    fn parent_info(&self) -> engine::ParentInfo {
        let parent = self.inner.as_ref().unwrap().clone();

        engine::ParentInfo {
            width: self.width,
            height: self.height,
            last_configure: 0,
            parent: PopupParent::LayerSurface(parent),
        }
    }

    fn set_instance(&mut self, instance: Arc<Mutex<engine::Instance>>) {
        self.instance = Some(instance);
    }

    fn instance(&mut self) -> Arc<Mutex<engine::Instance>> {
        if self.instance.is_none() {
            panic!("instance not set");
        }

        Arc::clone(&self.instance.as_ref().unwrap())
    }
}
