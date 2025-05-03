use std::{
    ptr::NonNull,
    sync::{Arc, Mutex},
    usize,
};

use std::sync::mpmc;

use raw_window_handle::{RawWindowHandle, WaylandWindowHandle};
use smithay_client_toolkit::{
    reexports::calloop::channel::Sender,
    shell::{
        xdg::{window::Window as XdgWindow, XdgSurface},
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

pub struct WindowInner {
    display_tx: Sender<Event>,

    backend: Backend,
    surface: Surface,

    tx: Sender<Request>,

    id: usize,

    title: String,
    app_id: String,
    width: usize,
    height: usize,

    window: Option<XdgWindow>,
}

#[derive(Clone)]
pub struct Window {
    inner: Arc<Mutex<WindowInner>>,
}

impl WindowInner {
    pub fn new(
        id: usize,
        display_tx: Sender<Event>,
        xdg_window: XdgWindow,
        backend: Backend,
        sm_tx: Sender<Request>,
    ) -> anyhow::Result<Self> {
        let ptr =
            NonNull::new(xdg_window.wl_surface().id().as_ptr() as *mut std::ffi::c_void).unwrap();
        let handle = WaylandWindowHandle::new(ptr);
        let surface = Surface::new(backend, &RawWindowHandle::Wayland(handle), 1, 1)?;

        Ok(Self {
            display_tx,

            id,
            surface,
            backend,
            tx: sm_tx,
            title: String::from(""),
            app_id: String::from(""),
            window: Some(xdg_window),
            width: 0,
            height: 0,
        })
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title.clone();
        self.window.as_ref().unwrap().set_title(title);
        self.window.as_ref().unwrap().commit();
    }

    pub fn set_app_id(&mut self, app_id: String) {
        self.app_id = app_id.clone();
        self.window.as_ref().unwrap().set_app_id(app_id);
        self.window.as_ref().unwrap().commit();
    }

    fn hide(&mut self) -> anyhow::Result<()> {
        self.surface.clear();

        let id = self.window.as_ref().unwrap().wl_surface().id();
        self.tx.send(Request::Unmap { id }).unwrap();

        self.window = None;

        Ok(())
    }

    fn show(&mut self) -> anyhow::Result<()> {
        let id = self.id();
        self.tx
            .send(Request::Recreate {
                id,
                req: RecreateRequest::Window,
            })
            .unwrap();

        Ok(())
    }
}

impl Window {
    pub fn new(
        app: &mut Application<'static>,
        display_tx: Sender<Event>,
        width: usize,
        height: usize,
    ) -> anyhow::Result<Self> {
        let sm = &mut app.lock().sm;

        let xdg_window = sm.create_xdg_window()?;
        xdg_window.set_min_size(Some((width as u32, height as u32)));
        xdg_window.commit();

        let id = sm.next_id();

        let surface_id = xdg_window.wl_surface().id();

        let inner = WindowInner::new(id, display_tx, xdg_window, sm.backend(), sm.tx())?;

        let window = Window {
            inner: Arc::new(Mutex::new(inner)),
        };

        sm.insert_state(id, &surface_id, window.inner.clone());

        Ok(window)
    }

    pub fn set_title(&mut self, title: impl Into<String>) {
        self.inner.lock().unwrap().set_title(title.into());
    }

    pub fn set_app_id(&mut self, app_id: impl Into<String>) {
        self.inner.lock().unwrap().set_app_id(app_id.into());
    }
}

impl State for WindowInner {
    fn id(&self) -> usize {
        self.id.clone()
    }

    fn map(&mut self, response: super::sm::RecreateResponse) {
        assert!(self.window.is_none());
        if let RecreateResponse::Window(window) = response {
            self.window = Some(window);
        }

        let ptr = NonNull::new(
            self.window.as_ref().unwrap().wl_surface().id().as_ptr() as *mut std::ffi::c_void
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

    fn pointer_axis(&mut self, horizontal: u64, vertical: u64, time: u32) {
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

impl Shell for Window {
    fn surface(&self) -> Surface {
        let inner = self.inner.lock().unwrap();
        inner.surface.clone()
    }

    fn parent_info(&self) -> engine::ParentInfo {
        let inner = self.inner.lock().unwrap();
        let xdg_surface = inner.window.as_ref().unwrap().xdg_surface().clone();

        engine::ParentInfo {
            width: inner.width,
            height: inner.height,
            last_configure: 0,
            parent: PopupParent::XdgSurface(xdg_surface),
        }
    }
}
