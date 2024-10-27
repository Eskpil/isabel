use cursor_icon::CursorIcon;
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{EventLoop, LoopHandle},
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
use xkeysym::{key::XF86_Close, Keysym};

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
    fn id(&self) -> ObjectId;
    fn surface(&self) -> &WlSurface;
    fn resize(&mut self, width: usize, height: usize, dx: usize, dy: usize);
    fn exit(&mut self);

    fn key_event(&mut self, event: &KeyEvent, state: KeyState);
    fn pointer_enter(&mut self, x: f64, y: f64);
    fn pointer_leave(&mut self);
    fn pointer_motion(&mut self, x: f64, y: f64, time: usize);
    fn pointer_button(&mut self, x: f64, y: f64, time: u32, button: PointerButtons, pressed: bool);
}

pub enum Request {
    Close { id: ObjectId },
    SetCursorIcon { icon: CursorIcon },
}

pub struct Application {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor_state: CompositorState,
    cursor_shape_manager: CursorShapeManager,

    loop_handle: LoopHandle<'static, Self>,
    qh: QueueHandle<Self>,

    keyboard: Option<wl_keyboard::WlKeyboard>,
    pointer: Option<wl_pointer::WlPointer>,

    xdg_shell: XdgShell,

    dph: WaylandDisplayHandle,

    tx: Sender<Request>,

    active_keyboard: Option<ObjectId>,

    last_enter_serial: u32,

    states: HashMap<ObjectId, Rc<RefCell<dyn State>>>,
}

impl Application {
    pub fn new(event_loop: &mut EventLoop<'static, Self>) -> anyhow::Result<Self> {
        let conn = Connection::connect_to_env().unwrap();
        let ptr = NonNull::new(conn.backend().display_ptr() as *mut std::ffi::c_void).unwrap();
        let dph = WaylandDisplayHandle::new(ptr);

        // Enumerate the list of globals to get the protocols the server implements.
        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh = event_queue.handle();

        let loop_handle = event_loop.handle();
        WaylandSource::new(conn.clone(), event_queue)
            .insert(loop_handle.clone())
            .unwrap();

        let (tx, rx) = channel();

        loop_handle
            .insert_source(rx, |e, _, a| {
                if let Event::Msg(req) = e {
                    match req {
                        Request::Close { id } => {
                            a.states.remove(&id);
                        }
                        Request::SetCursorIcon { icon } => {
                            let shape_device = a
                                .cursor_shape_manager
                                .get_shape_device(&a.pointer.as_ref().unwrap(), &a.qh);
                            shape_device
                                .set_shape(a.last_enter_serial, util::cursor_icon_to_shape(icon));
                        }
                    }
                }
            })
            .expect("could not insert channel");

        Ok(Self {
            loop_handle,

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
            tx,
            dph,

            states: HashMap::new(),
        })
    }

    pub fn exited(&self) -> bool {
        self.states.is_empty()
    }

    pub fn create_window(
        &mut self,
        title: &str,
        width: usize,
        height: usize,
    ) -> anyhow::Result<Rc<RefCell<Window>>> {
        let surface = self.compositor_state.create_surface(&self.qh);
        let id = surface.id();
        let xdg_window = self
            .xdg_shell
            .create_window(surface, WindowDecorations::None, &self.qh);

        xdg_window.set_title(title);
        xdg_window.set_app_id("io.isabel.example");
        xdg_window.set_min_size(Some((256, 256)));
        xdg_window.commit();

        let mut backend = Backend::new(&RawDisplayHandle::Wayland(self.dph))?;

        {
            let ptr = NonNull::new(xdg_window.wl_surface().id().as_ptr() as *mut std::ffi::c_void)
                .unwrap();
            let handle = WaylandWindowHandle::new(ptr);
            _ = backend.surface(&RawWindowHandle::Wayland(handle), width, height)?;
        };

        let window = Window::new(self.tx.clone(), Arc::new(Mutex::new(backend)), xdg_window);
        let window = Rc::new(RefCell::new(window));

        self.states.insert(id, window.clone());

        Ok(window)
    }
}

impl CompositorHandler for Application {
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

impl OutputHandler for Application {
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

impl WindowHandler for Application {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &XdgWindow) {
        {
            let mut state = self
                .states
                .get_mut(&window.wl_surface().id())
                .expect("no state for surface")
                .borrow_mut();
            state.exit();
        }
        self.states.remove(&window.wl_surface().id());
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        window: &XdgWindow,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        let mut state = self
            .states
            .get_mut(&window.wl_surface().id())
            .expect("no state for surface")
            .borrow_mut();

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

impl SeatHandler for Application {
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
                            let state = app.states.get_mut(&id).expect("no state for surface");
                            state.borrow_mut().key_event(&event, KeyState::Pressed);
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

impl KeyboardHandler for Application {
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
                let state = self.states.get_mut(&id).expect("no state for surface");
                state.borrow_mut().key_event(&event, KeyState::Pressed);
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

impl PointerHandler for Application {
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
            let state = self
                .states
                .get_mut(&event.surface.id())
                .expect("no state for surface");

            match &event.kind {
                Enter { serial } => {
                    self.last_enter_serial = *serial;
                    state
                        .borrow_mut()
                        .pointer_enter(event.position.0, event.position.0);
                }
                Leave { .. } => state.borrow_mut().pointer_leave(),
                Motion { time } => state.borrow_mut().pointer_motion(
                    event.position.0,
                    event.position.1,
                    *time as usize,
                ),
                Press {
                    time,
                    button,
                    serial,
                } => {
                    state.borrow_mut().pointer_button(
                        event.position.0,
                        event.position.1,
                        *time,
                        map_event_code(*button),
                        true,
                    );
                }
                Release {
                    time,
                    button,
                    serial,
                } => state.borrow_mut().pointer_button(
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

delegate_compositor!(Application);
delegate_output!(Application);

delegate_seat!(Application);
delegate_keyboard!(Application);
delegate_pointer!(Application);

delegate_xdg_shell!(Application);
delegate_xdg_window!(Application);

delegate_registry!(Application);

impl ProvidesRegistryState for Application {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState, SeatState,];
}
