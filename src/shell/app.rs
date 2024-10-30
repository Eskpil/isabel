use cursor_icon::CursorIcon;
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_xdg_popup, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::LoopHandle,
        calloop_wayland_source::WaylandSource,
        client::{
            protocol::{wl_keyboard, wl_pointer},
            Connection,
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Modifiers},
        pointer::{
            cursor_shape::CursorShapeManager, PointerEvent, PointerEventKind, PointerHandler,
        },
        Capability, SeatHandler, SeatState,
    },
    shell::{
        xdg::{
            popup::{Popup as ToolkitPopup, PopupHandler},
            window::{Window as XdgWindow, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
        },
        WaylandSurface,
    },
};
use wayland_backend::client::ObjectId;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{
        wl_output, wl_seat,
        wl_surface::{self, WlSurface},
    },
    Proxy, QueueHandle,
};
use xkeysym::Keysym;

use std::{
    cell::RefCell,
    collections::HashMap,
    ptr::NonNull,
    rc::Rc,
    sync::{Arc, Mutex},
};

use super::{
    channel::{channel, Event, Sender},
    util,
    window::Window,
};

use crate::{backend::Backend, engine::PointerButtons};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum KeyState {
    Released,
    Pressed,
}

pub trait State {
    fn id(&self) -> usize;
    fn surface(&self) -> &WlSurface;
    fn resize(&mut self, width: usize, height: usize, dx: usize, dy: usize);
    fn exit(&mut self);

    fn map(&mut self, _response: RecreateResponse) {}

    fn key_event(&mut self, event: &KeyEvent, state: KeyState);
    fn pointer_enter(&mut self, x: f64, y: f64);
    fn pointer_leave(&mut self);
    fn pointer_motion(&mut self, x: f64, y: f64, time: usize);
    fn pointer_button(&mut self, x: f64, y: f64, time: u32, button: PointerButtons, pressed: bool);
}

pub enum RecreateRequest {
    Window,
}

pub enum RecreateResponse {
    Window(XdgWindow),
}

pub enum Request {
    Recreate { id: usize, req: RecreateRequest },
    Unmap { id: ObjectId },
    SetCursorIcon { icon: CursorIcon },
}

pub struct Application<'a> {
    pub(crate) registry_state: RegistryState,
    pub(crate) seat_state: SeatState,
    pub(crate) output_state: OutputState,
    pub(crate) compositor_state: CompositorState,
    pub(crate) cursor_shape_manager: CursorShapeManager,

    pub(crate) loop_handle: LoopHandle<'a, Self>,
    pub(crate) qh: QueueHandle<Self>,

    pub(crate) keyboard: Option<wl_keyboard::WlKeyboard>,
    pub(crate) pointer: Option<wl_pointer::WlPointer>,

    pub(crate) xdg_shell: XdgShell,

    pub(crate) dph: WaylandDisplayHandle,
    pub(crate) tx: Sender<Request>,

    pub(crate) active_keyboard: Option<ObjectId>,

    pub(crate) last_enter_serial: u32,

    counter: usize,

    pub(crate) states: HashMap<usize, Rc<RefCell<dyn State>>>,
    pub(crate) surface_id_to_id: HashMap<ObjectId, usize>,
}

impl<'a> Application<'a>
where
    'a: 'static,
{
    pub fn new(handle: LoopHandle<'a, Self>) -> anyhow::Result<Self> {
        let conn = Connection::connect_to_env().unwrap();
        let ptr = NonNull::new(conn.backend().display_ptr() as *mut std::ffi::c_void).unwrap();
        let dph = WaylandDisplayHandle::new(ptr);

        // Enumerate the list of globals to get the protocols the server implements.
        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh = event_queue.handle();

        let source = WaylandSource::new(conn.clone(), event_queue);
        handle
            .clone()
            .insert_source(source, |_, queue, data| queue.dispatch_pending(data))
            .expect("could not insert");

        let (tx, rx) = channel();

        handle
            .insert_source(rx, |e, _, a| {
                if let Event::Msg(req) = e {
                    match req {
                        Request::Recreate { id, req } => {
                            let surface = a.compositor_state.create_surface(&a.qh);
                            let surface_id = surface.id();

                            let response = match req {
                                RecreateRequest::Window => {
                                    let xdg_window = a.xdg_shell.create_window(
                                        surface,
                                        WindowDecorations::None,
                                        &a.qh,
                                    );

                                    xdg_window.set_min_size(Some((256, 256)));
                                    xdg_window.commit();

                                    RecreateResponse::Window(xdg_window)
                                }
                            };

                            a.surface_id_to_id.insert(surface_id.clone(), id);
                            let state = a.state_mut(&surface_id);
                            let mut state = state.borrow_mut();
                            state.map(response);
                        }
                        Request::Unmap { id } => {
                            a.surface_id_to_id.remove(&id);
                        }
                        Request::SetCursorIcon { icon } => {
                            a.set_cursor_icon(icon).expect("could not set cursor icon");
                        }
                    }
                }
            })
            .unwrap();

        Ok(Self {
            loop_handle: handle,

            active_keyboard: None,

            keyboard: None,
            pointer: None,

            compositor_state: CompositorState::bind(&globals, &qh)?,
            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),

            cursor_shape_manager: CursorShapeManager::bind(&globals, &qh)?,
            last_enter_serial: 0,

            xdg_shell: XdgShell::bind(&globals, &qh)?,

            qh,
            dph,
            tx,

            counter: 0,

            states: HashMap::new(),
            surface_id_to_id: HashMap::new(),
        })
    }

    pub fn state_mut(&mut self, id: &ObjectId) -> Rc<RefCell<dyn State>> {
        let id = self.surface_id_to_id[id];
        self.states[&id].clone()
    }

    pub fn has_state(&mut self, id: &ObjectId) -> bool {
        self.surface_id_to_id.contains_key(id)
    }

    pub fn exited(&self) -> bool {
        self.states.is_empty()
    }

    pub fn set_cursor_icon(&self, icon: CursorIcon) -> anyhow::Result<()> {
        let shape_device = self
            .cursor_shape_manager
            .get_shape_device(&self.pointer.as_ref().unwrap(), &self.qh);

        shape_device.set_shape(self.last_enter_serial, util::cursor_icon_to_shape(icon));
        Ok(())
    }

    pub fn create_window(
        &mut self,
        width: usize,
        height: usize,
    ) -> anyhow::Result<Rc<RefCell<Window>>> {
        let surface = self.compositor_state.create_surface(&self.qh);
        let surface_id = surface.id();
        let xdg_window = self
            .xdg_shell
            .create_window(surface, WindowDecorations::None, &self.qh);

        xdg_window.set_min_size(Some((width as u32, height as u32)));
        xdg_window.commit();

        let mut backend = Backend::new(&RawDisplayHandle::Wayland(self.dph))?;

        {
            let ptr = NonNull::new(xdg_window.wl_surface().id().as_ptr() as *mut std::ffi::c_void)
                .unwrap();
            let handle = WaylandWindowHandle::new(ptr);
            _ = backend.surface(&RawWindowHandle::Wayland(handle), width, height)?;
        };

        let tx = self.tx.clone();
        let id = self.counter.clone();

        let window = Window::new(id, tx, Arc::new(Mutex::new(backend)), xdg_window);
        let window = Rc::new(RefCell::new(window));

        self.states.insert(id, window.clone());
        self.surface_id_to_id.insert(surface_id, id);
        self.counter += 1;

        Ok(window)
    }
}

impl<'a> CompositorHandler for Application<'a>
where
    'a: 'static,
{
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Not needed for this example.
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
    }
}

impl<'a> OutputHandler for Application<'a>
where
    'a: 'static,
{
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl<'a> WindowHandler for Application<'a>
where
    'a: 'static,
{
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &XdgWindow) {
        if !self.has_state(&window.wl_surface().id()) {
            return;
        }

        let id = {
            let state = self.state_mut(&window.wl_surface().id());
            let mut state = state.borrow_mut();
            state.exit();
            state.id()
        };

        self.surface_id_to_id.remove(&window.wl_surface().id());
        self.states.remove(&id);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        window: &XdgWindow,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        if !self.has_state(&window.wl_surface().id()) {
            return;
        }

        let state = self.state_mut(&window.wl_surface().id());
        let mut state = state.borrow_mut();

        let width = configure.new_size.0.map(|v| v.get()).unwrap_or(256);
        let height = configure.new_size.1.map(|v| v.get()).unwrap_or(256);

        let mut update_dimensions = || {
            state.resize(width as usize, height as usize, 0, 0);
        };

        // FIXME: Seems like the first state is always empty. So, we have to kickstart it. Most likely
        //        smithay-client-toolkit is at fault.
        if configure.state.is_empty() {
            update_dimensions();
        }

        if configure.is_resizing()
            || configure.is_tiled()
            || configure.is_maximized()
            || configure.is_fullscreen()
        {
            update_dimensions();
        }

        // activated events seems to be returned after the window is tiled.
        if configure.is_activated() {
            update_dimensions();
        }
    }
}

impl<'a> SeatHandler for Application<'a>
where
    'a: 'static,
{
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            let keyboard = self
                .seat_state
                .get_keyboard_with_repeat(
                    qh,
                    &seat,
                    None,
                    self.loop_handle.clone(),
                    Box::new(move |app, _, event| match app.active_keyboard.clone() {
                        Some(id) => {
                            if !app.has_state(&id) {
                                return;
                            }
                            let state = app.state_mut(&id);
                            let mut state = state.borrow_mut();
                            state.key_event(&event, KeyState::Pressed);
                        }
                        None => {}
                    }),
                )
                .expect("Failed to create keyboard");

            self.keyboard = Some(keyboard);
        }

        if capability == Capability::Pointer && self.pointer.is_none() {
            let pointer = self
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to create pointer");

            self.pointer = Some(pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        _capability: Capability,
    ) {
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl<'a> KeyboardHandler for Application<'a>
where
    'a: 'static,
{
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _keysyms: &[Keysym],
    ) {
        self.active_keyboard = Some(surface.id())
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        self.active_keyboard = None;
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        match self.active_keyboard.clone() {
            Some(id) => {
                if !self.has_state(&id) {
                    return;
                }

                let state = self.state_mut(&id);
                let mut state = state.borrow_mut();
                state.key_event(&event, KeyState::Pressed);
            }
            None => {}
        };
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        _event: KeyEvent,
    ) {
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _modifiers: Modifiers,
        _layout: u32,
    ) {
    }
}

impl<'a> PointerHandler for Application<'a>
where
    'a: 'static,
{
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        let map_event_code = |button| match button {
            linux_event_codes::BTN_LEFT => PointerButtons::Primary,
            linux_event_codes::BTN_RIGHT => PointerButtons::Secondary,
            linux_event_codes::BTN_MIDDLE => PointerButtons::Middle,
            linux_event_codes::BTN_BACK => PointerButtons::Back,

            o => panic!("not a mouse button: {:?}", o),
        };

        use PointerEventKind::*;
        for event in events {
            if !self.has_state(&event.surface.id()) {
                continue;
            }

            let state = self.state_mut(&event.surface.id());
            let mut state = state.borrow_mut();

            match &event.kind {
                Enter { serial } => {
                    self.last_enter_serial = *serial;
                    state.pointer_enter(event.position.0, event.position.0);
                }
                Leave { .. } => state.pointer_leave(),
                Motion { time } => {
                    state.pointer_motion(event.position.0, event.position.1, *time as usize)
                }
                Press { time, button, .. } => {
                    state.pointer_button(
                        event.position.0,
                        event.position.1,
                        *time,
                        map_event_code(*button),
                        true,
                    );
                }
                Release { time, button, .. } => state.pointer_button(
                    event.position.0,
                    event.position.1,
                    *time,
                    map_event_code(*button),
                    false,
                ),

                o => todo!("implement pointer event: {:?}", o),
            };
        }
    }
}

impl<'a> PopupHandler for Application<'a> {
    fn done(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _popup: &ToolkitPopup) {
        println!("popup done");
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _popup: &ToolkitPopup,
        _config: smithay_client_toolkit::shell::xdg::popup::PopupConfigure,
    ) {
        println!("got popup configure");
    }
}

delegate_compositor!(@<'a: 'static> Application<'a>);
delegate_output!(@<'a: 'static>Application<'a>);

delegate_seat!(@<'a: 'static>Application<'a>);
delegate_keyboard!(@<'a: 'static>Application<'a>);
delegate_pointer!(@<'a: 'static>Application<'a>);

delegate_xdg_shell!(@<'a: 'static>Application<'a>);
delegate_xdg_window!(@<'a: 'static>Application<'a>);

delegate_registry!(@<'a: 'static>Application<'a>);
delegate_xdg_popup!(@<'a: 'static>Application<'a>);

impl<'a> ProvidesRegistryState for Application<'a>
where
    'a: 'static,
{
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState, SeatState,];
}
