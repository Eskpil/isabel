use std::sync::{Arc, Mutex};

use cursor_icon::CursorIcon;
use smithay_client_toolkit::{
    reexports::calloop::channel::Sender,
    shell::{xdg::window::Window as XdgWindow, WaylandSurface},
};
use wayland_backend::client::ObjectId;
use wayland_client::{protocol::wl_surface::WlSurface, Proxy};

use super::app::Request;
use crate::{
    backend::Backend,
    engine::{self, PointerButtons},
};

use super::app::{KeyState, State};

pub struct Window {
    backend: Arc<Mutex<Backend>>,
    instance: Option<engine::Instance>,
    tx: Sender<Request>,

    window: XdgWindow,
}

impl Window {
    pub fn new(tx: Sender<Request>, backend: Arc<Mutex<Backend>>, window: XdgWindow) -> Self {
        Self {
            tx,
            backend,
            window,
            instance: None,
        }
    }
}

impl State for Window {
    fn id(&self) -> ObjectId {
        self.window.wl_surface().id()
    }

    fn exit(&mut self) {
        self.instance.as_mut().unwrap().engine_mut().exit();
    }

    fn surface(&self) -> &WlSurface {
        self.window.wl_surface()
    }

    fn resize(&mut self, width: usize, height: usize, dx: usize, dy: usize) {
        self.backend
            .lock()
            .unwrap()
            .resize(0, width, height, dx, dy);
        self.instance
            .as_mut()
            .unwrap()
            .engine_mut()
            .resize(width, height)
            .unwrap();
    }

    fn pointer_enter(&mut self, x: f64, y: f64) {
        self.instance
            .as_mut()
            .unwrap()
            .engine_mut()
            .pointer_enter(x, y)
            .unwrap();
    }

    fn pointer_leave(&mut self) {
        self.instance
            .as_mut()
            .unwrap()
            .engine_mut()
            .pointer_leave()
            .unwrap();
    }

    fn pointer_motion(&mut self, x: f64, y: f64, time: usize) {
        self.instance
            .as_mut()
            .unwrap()
            .engine_mut()
            .pointer_motion(x, y, time)
            .unwrap();
    }

    fn pointer_button(&mut self, x: f64, y: f64, time: u32, button: PointerButtons, pressed: bool) {
        self.instance
            .as_mut()
            .unwrap()
            .engine_mut()
            .pointer_button(x, y, time, button, pressed)
            .unwrap();
    }

    fn key_event(
        &mut self,
        event: &smithay_client_toolkit::seat::keyboard::KeyEvent,
        state: KeyState,
    ) {
        if state == KeyState::Pressed {
            self.instance
                .as_mut()
                .unwrap()
                .engine_mut()
                .key_press(event.keysym)
                .unwrap();
        }
    }
}

impl engine::Shell for Window {
    fn backend(&mut self) -> anyhow::Result<Arc<Mutex<Backend>>> {
        Ok(self.backend.clone())
    }

    fn set_cursor_icon(&mut self, icon: CursorIcon) -> anyhow::Result<()> {
        self.tx.send(Request::SetCursorIcon { icon })?;
        Ok(())
    }

    fn capabilities(&self) -> engine::ShellCapabilities {
        engine::ShellCapabilities::all()
    }

    fn set_instance(&mut self, instance: engine::Instance) {
        self.instance = Some(instance);
    }

    fn instance_mut(&mut self) -> &mut engine::Instance {
        if self.instance.is_none() {
            panic!("instance not set");
        }

        self.instance.as_mut().unwrap()
    }
}
