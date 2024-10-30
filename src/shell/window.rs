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
    shell::{xdg::window::Window as XdgWindow, WaylandSurface},
};
use wayland_client::{protocol::wl_surface::WlSurface, Proxy};

use crate::{
    backend::{Backend, MAIN_SURFACE},
    engine::{self, PointerButtons, Shell},
    shell::app::Request,
};

use super::app::{KeyState, RecreateRequest, RecreateResponse, State};

pub struct Window {
    backend: Arc<Mutex<Backend>>,
    instance: Option<Rc<RefCell<engine::Instance>>>,
    tx: Sender<Request>,

    id: usize,

    title: String,
    app_id: String,
    width: usize,
    height: usize,

    window: Option<XdgWindow>,
}

impl Window {
    pub fn new(
        id: usize,
        tx: Sender<Request>,
        backend: Arc<Mutex<Backend>>,
        window: XdgWindow,
    ) -> Self {
        Self {
            id,
            backend,
            tx,
            title: String::from(""),
            app_id: String::from(""),
            window: Some(window),
            instance: None,
            width: 0,
            height: 0,
        }
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
}

impl State for Window {
    fn id(&self) -> usize {
        self.id.clone()
    }

    fn map(&mut self, response: super::app::RecreateResponse) {
        assert!(self.window.is_none());
        if let RecreateResponse::Window(window) = response {
            self.window = Some(window);
        }

        let ptr = NonNull::new(
            self.window.as_ref().unwrap().wl_surface().id().as_ptr() as *mut std::ffi::c_void
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
        self.window.as_ref().unwrap().wl_surface()
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

impl Shell for Window {
    fn backend(&mut self) -> anyhow::Result<Arc<Mutex<Backend>>> {
        Ok(self.backend.clone())
    }

    fn hide(&mut self) -> anyhow::Result<()> {
        self.instance().borrow_mut().hide();
        self.pointer_leave();

        self.backend.lock().unwrap().remove(&MAIN_SURFACE);

        let id = self.surface().id();
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
