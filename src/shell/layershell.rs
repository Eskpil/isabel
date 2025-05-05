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
    Application,
};

use super::{
    event::Event,
    sm::{KeyState, PopupParent, RecreateRequest, RecreateResponse, Request, State},
};

pub(crate) struct LayerSurfaceInner {
    display_tx: Sender<Event>,

    backend: Backend,
    surface: Surface,
    instance: Option<Arc<Mutex<engine::Instance>>>,
    tx: Sender<Request>,

    id: usize,

    width: usize,
    height: usize,

    inner: Option<WlrLayerSurface>,
}

pub struct LayerSurface {
    inner: Arc<Mutex<LayerSurfaceInner>>,
}

impl LayerSurface {
    pub fn new(
        app: &mut Application<'static>,
        display_tx: Sender<Event>,
        layer: Layer,
        namespace: impl Into<String>,
    ) -> anyhow::Result<Self> {
        let sm = &mut app.lock().sm;
        let wlr_layer_surface = sm.create_layer_surface(layer, namespace.into())?;

        let id = sm.next_id();
        let surface_id = wlr_layer_surface.wl_surface().id();

        let inner =
            LayerSurfaceInner::new(id, sm.tx(), display_tx, sm.backend(), wlr_layer_surface)?;

        let layer_surface = Self {
            inner: Arc::new(Mutex::new(inner)),
        };

        sm.insert_state(id, &surface_id, layer_surface.inner.clone());

        Ok(layer_surface)
    }

    pub fn set_anchor(&mut self, anchor: Anchor) {
        self.inner.lock().unwrap().set_anchor(anchor);
    }

    pub fn set_exclusive_zone(&mut self, zone: i32) {
        self.inner.lock().unwrap().set_exclusive_zone(zone);
    }

    pub fn set_keyboard_interactivity(&mut self, value: KeyboardInteractivity) {
        self.inner.lock().unwrap().set_keyboard_interactivity(value);
    }

    pub fn set_layer(&mut self, layer: Layer) {
        self.inner.lock().unwrap().set_layer(layer);
    }

    pub fn set_size(&mut self, width: usize, height: usize) {
        self.inner.lock().unwrap().set_size(width, height);
    }

    pub fn commit(&mut self) {
        self.inner.lock().unwrap().commit();
    }

    fn hide(&mut self) -> anyhow::Result<()> {
        self.inner.lock().unwrap().hide()
    }

    fn show(&mut self) -> anyhow::Result<()> {
        self.inner.lock().unwrap().show()
    }
}

impl LayerSurfaceInner {
    pub fn new(
        id: usize,
        sm_tx: Sender<Request>,
        display_tx: Sender<Event>,
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
            tx: sm_tx,
            display_tx,
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
        self.display_tx.send(Event::Hide).unwrap();
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

        self.display_tx.send(Event::Show).unwrap();

        Ok(())
    }
}

impl State for LayerSurfaceInner {
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
        self.display_tx.send(Event::Exit).unwrap();
    }

    fn resize(&mut self, width: usize, height: usize, dx: usize, dy: usize) {
        self.width = width;
        self.height = height;

        self.surface
            .resize(width, height, dx, dy)
            .expect("could not resize");

        self.display_tx
            .send(Event::Resize {
                width,
                height,
                scale: 1 as f64,
            })
            .unwrap();
    }

    fn pointer_enter(&mut self, x: f64, y: f64) {
        self.display_tx.send(Event::PointerEnter { x, y }).unwrap();
    }

    fn pointer_leave(&mut self) {
        self.display_tx.send(Event::PointerLeave {}).unwrap();
    }

    fn pointer_motion(&mut self, x: f64, y: f64, time: usize) {
        self.display_tx
            .send(Event::PointerMotion {
                x,
                y,
                time: time as u64,
            })
            .unwrap();
    }

    fn pointer_button(
        &mut self,
        x: f64,
        y: f64,
        time: u32,
        button: PointerButtons,
        state: KeyState,
    ) {
        self.display_tx
            .send(Event::PointerButton {
                state,
                x,
                y,
                time: time as u64,
                button,
            })
            .unwrap();
    }

    fn pointer_axis(&mut self, horizontal: f64, vertical: f64, time: u32) {
        self.display_tx
            .send(Event::PointerAxis {
                horizontal,
                vertical,
                time: time as u64,
            })
            .unwrap();
    }

    fn key_event(
        &mut self,
        event: &smithay_client_toolkit::seat::keyboard::KeyEvent,
        state: KeyState,
    ) {
        self.display_tx
            .send(Event::Key {
                state,
                symbol: event.keysym,
                time: event.time as u64,
            })
            .unwrap();
    }
}

impl Shell for LayerSurface {
    fn surface(&self) -> Surface {
        let inner = self.inner.lock().unwrap();
        inner.surface.clone()
    }

    fn parent_info(&self) -> engine::ParentInfo {
        let inner = self.inner.lock().unwrap();
        let parent = inner.inner.as_ref().unwrap().clone();

        engine::ParentInfo {
            width: inner.width,
            height: inner.height,
            last_configure: 0,
            parent: PopupParent::LayerSurface(parent),
        }
    }
}
