use std::ptr::NonNull;

use backend::{self, Backend};
use engine::{Event, PointerButtons};

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
        pointer::cursor_shape::CursorShapeManager,
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
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
use smithay_client_toolkit::{
    reexports::{
        calloop::{channel::Sender, EventLoop, LoopHandle},
        calloop_wayland_source::WaylandSource,
        client::{
            globals::registry_queue_init,
            protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface},
            Connection, QueueHandle,
        },
    },
    seat::pointer::CursorIcon,
};
use wayland_client::Proxy;

use crate::util;

pub struct Window {
    tx: Sender<Event>,

    title: String,
    width: u32,
    height: u32,

    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,

    loop_handle: LoopHandle<'static, Self>,

    conn: Connection,
    exit: bool,
    shift: Option<u32>,
    window: XdgWindow,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    keyboard_focus: bool,
    pointer: Option<wl_pointer::WlPointer>,

    cursor_shape_mgr: CursorShapeManager,

    egl_surface: wayland_egl::WlEglSurface,

    last_enter_serial: u32,
    last_pointer_serial: u32,

    qh: QueueHandle<Window>,

    instance: Option<engine::Instance>,
}

impl Window {
    pub fn create(
        tx: Sender<Event>,
        title: &str,
        width: u32,
        height: u32,
        event_loop: &mut EventLoop<'static, Self>,
    ) -> anyhow::Result<Self> {
        let conn = Connection::connect_to_env().unwrap();

        // Enumerate the list of globals to get the protocols the server implements.
        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh = event_queue.handle();

        let loop_handle = event_loop.handle();
        WaylandSource::new(conn.clone(), event_queue)
            .insert(loop_handle.clone())
            .unwrap();

        let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
        let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell is not available");

        let surface = compositor.create_surface(&qh);

        let wl_egl_surface =
            wayland_egl::WlEglSurface::new(surface.id(), width as i32, height as i32)?;

        let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);
        window.set_title("A wayland window");
        window.set_app_id("io.github.smithay.client-toolkit.SimpleWindow");
        window.set_min_size(Some((256, 256)));

        window.commit();

        Ok(Self {
            tx,

            loop_handle,

            title: title.into(),
            width,
            height,

            conn,

            exit: false,
            keyboard: None,
            shift: None,
            window,
            keyboard_focus: false,
            pointer: None,

            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),

            cursor_shape_mgr: CursorShapeManager::bind(&globals, &qh)?,

            egl_surface: wl_egl_surface,

            last_enter_serial: 0,
            last_pointer_serial: 0,

            qh,

            instance: None,
        })
    }
}

impl engine::Shell for Window {
    fn backend(&mut self) -> anyhow::Result<backend::Backend> {
        let mut backend = {
            log::info!("trying to make backend");
            let ptr =
                NonNull::new(self.conn.backend().display_ptr() as *mut std::ffi::c_void).unwrap();
            let handle = WaylandDisplayHandle::new(ptr);
            Backend::new(&RawDisplayHandle::Wayland(handle))?
        };

        {
            let ptr = NonNull::new(self.egl_surface.ptr() as *mut std::ffi::c_void).unwrap();
            let handle = WaylandWindowHandle::new(ptr);
            let _ = backend.surface(&RawWindowHandle::Wayland(handle))?;
        }

        Ok(backend)
    }

    fn set_cursor_icon(&mut self, cursor: CursorIcon) -> anyhow::Result<()> {
        let pointer = self.pointer.as_ref().unwrap();
        let device = self.cursor_shape_mgr.get_shape_device(pointer, &self.qh);
        device.set_shape(self.last_enter_serial, util::cursor_icon_to_shape(cursor));
        Ok(())
    }

    fn capabilities(&self) -> engine::ShellCapabilities {
        engine::ShellCapabilities::all()
    }

    fn initiate_move(&mut self, serial: u32) {
        println!("window: initiate_move: ({})", serial);

        self.window.move_(
            &self.seat_state.seats().last().unwrap(),
            self.last_pointer_serial,
        );
    }

    fn exited(&self) -> bool {
        self.exit
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

impl CompositorHandler for Window {
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

impl OutputHandler for Window {
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

impl WindowHandler for Window {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &XdgWindow) {
        self.exit = true;
        if self.exit {
            self.tx.send(Event::Close).expect("could not send");
        }
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _window: &XdgWindow,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        self.width = configure.new_size.0.map(|v| v.get()).unwrap_or(256);
        self.height = configure.new_size.1.map(|v| v.get()).unwrap_or(256);

        let update_dimensions = || {
            self.egl_surface
                .resize(self.width as i32, self.height as i32, 0, 0);

            self.tx
                .send(Event::Resized {
                    width: self.width,
                    height: self.height,
                })
                .expect("could not send");
            ()
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
            // update_dimensions();
        }
    }
}

impl SeatHandler for Window {
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
            let tx = self.tx.clone();
            let keyboard = self
                .seat_state
                .get_keyboard_with_repeat(
                    qh,
                    &seat,
                    None,
                    self.loop_handle.clone(),
                    Box::new(move |_state, _wl_kbd, event| {
                        tx.send(engine::Event::Keypress {
                            symbol: event.keysym,
                            pressed: true,
                        })
                        .unwrap();
                    }),
                )
                .expect("Failed to create keyboard");

            self.keyboard = Some(keyboard);

            let keyboard = self
                .seat_state
                .get_keyboard(qh, &seat, None)
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
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_some() {
            self.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.pointer.is_some() {
            self.pointer.take().unwrap().release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for Window {
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
        if self.window.wl_surface() == surface {
            self.keyboard_focus = true;
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        if self.window.wl_surface() == surface {
            self.keyboard_focus = false;
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        self.tx
            .send(engine::Event::Keypress {
                symbol: event.keysym,
                pressed: true,
            })
            .unwrap();
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

impl PointerHandler for Window {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        let button_event = |x, y, button, time, serial, pressed| {
            let button = match button {
                linux_event_codes::BTN_LEFT => PointerButtons::Primary,
                linux_event_codes::BTN_RIGHT => PointerButtons::Secondary,
                linux_event_codes::BTN_MIDDLE => PointerButtons::Middle,
                linux_event_codes::BTN_BACK => PointerButtons::Back,

                o => panic!("not a mouse button: {:?}", o),
            };

            engine::Event::PointerButton {
                x,
                y,
                button,
                time,
                pressed,
                serial,
            }
        };

        use PointerEventKind::*;
        for event in events {
            // Ignore events for other surfaces
            if &event.surface != self.window.wl_surface() {
                continue;
            }

            let event = match &event.kind {
                Enter { serial } => {
                    self.last_enter_serial = *serial;
                    engine::Event::PointerEnter {
                        x: event.position.0,
                        y: event.position.1,
                    }
                }
                Leave { .. } => engine::Event::PointerLeave {
                    x: event.position.0,
                    y: event.position.1,
                },
                Motion { time } => engine::Event::PointerMotion {
                    x: event.position.0,
                    y: event.position.1,
                    time: *time,
                    serial: self.last_pointer_serial,
                },
                Press {
                    time,
                    button,
                    serial,
                } => {
                    let e = button_event(
                        event.position.0,
                        event.position.1,
                        *button,
                        *time,
                        *serial,
                        true,
                    );

                    if *button == linux_event_codes::BTN_LEFT {
                        self.last_pointer_serial = *serial;
                        println!("last set serial: {}", *serial);
                    }

                    e
                }
                Release {
                    time,
                    button,
                    serial,
                } => button_event(
                    event.position.0,
                    event.position.1,
                    *button,
                    *time,
                    *serial,
                    false,
                ),

                o => todo!("implement pointer event: {:?}", o),
            };

            self.tx.send(event).expect("could not send");
        }
    }
}

delegate_compositor!(Window);
delegate_output!(Window);

delegate_seat!(Window);
delegate_keyboard!(Window);
delegate_pointer!(Window);

delegate_xdg_shell!(Window);
delegate_xdg_window!(Window);

delegate_registry!(Window);

impl ProvidesRegistryState for Window {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState, SeatState,];
}
