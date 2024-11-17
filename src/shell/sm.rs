use cursor_icon::CursorIcon;
use raw_window_handle::{RawDisplayHandle, WaylandDisplayHandle};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_xdg_popup, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::LoopHandle,
        calloop_wayland_source::WaylandSource,
        client::{
            protocol::{wl_keyboard, wl_pointer},
            Connection,
        },
        protocols::xdg::shell::client::xdg_surface::XdgSurface,
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
        wlr_layer::{LayerShell, LayerShellHandler, LayerSurface as WlrLayerSurface},
        xdg::{
            popup::{Popup as ToolkitPopup, PopupHandler},
            window::{Window as XdgWindow, WindowConfigure, WindowDecorations, WindowHandler},
            XdgPositioner, XdgShell,
        },
        WaylandSurface,
    },
};
use wayland_backend::client::ObjectId;
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_output, wl_seat, wl_surface},
    Proxy, QueueHandle,
};
use xkeysym::Keysym;

use std::{
    collections::HashMap,
    ptr::NonNull,
    sync::{Arc, Mutex},
};

use super::{
    channel::{channel, Event, Sender},
    layershell::LayerSurface,
    popup::Popup,
    positioner::Positioner,
    util,
    window::Window,
};

use crate::{app::Application, backend::Backend, engine::PointerButtons};

#[derive(Debug)]
pub enum PopupParent {
    XdgSurface(XdgSurface),
    LayerSurface(WlrLayerSurface),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum KeyState {
    Released,
    Pressed,
}

pub trait State {
    fn id(&self) -> usize;
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
    Layershell,
    Popup(Positioner, PopupParent),
}

pub enum RecreateResponse {
    Window(XdgWindow),
    Layershell(WlrLayerSurface),
    Popup(ToolkitPopup),
}

pub enum Request {
    Recreate { id: usize, req: RecreateRequest },
    Unmap { id: ObjectId },
    PopupGrab { popup: ToolkitPopup, serial: usize },
}

pub struct SurfaceManager<'a> {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor_state: CompositorState,
    cursor_shape_manager: CursorShapeManager,

    backend: Backend,

    qh: QueueHandle<Self>,

    keyboard: Option<wl_keyboard::WlKeyboard>,
    pointer: Option<wl_pointer::WlPointer>,

    xdg_shell: XdgShell,
    layershell: LayerShell,

    tx: Sender<Request>,

    active_keyboard: Option<ObjectId>,

    last_enter_serial: u32,

    counter: usize,

    states: HashMap<usize, Arc<Mutex<dyn State>>>,
    surface_id_to_id: HashMap<ObjectId, usize>,
}

impl<'a> SurfaceManager<'a>
where
    'a: 'static,
{
    pub fn new(handle: LoopHandle<'a, Application<'a>>) -> anyhow::Result<Self> {
        let conn = Connection::connect_to_env().unwrap();
        let ptr = NonNull::new(conn.backend().display_ptr() as *mut std::ffi::c_void).unwrap();
        let dph = WaylandDisplayHandle::new(ptr);

        let backend = Backend::global_prepare(&RawDisplayHandle::Wayland(dph))?;

        // Enumerate the list of globals to get the protocols the server implements.
        let (globals, event_queue) = registry_queue_init::<SurfaceManager<'a>>(&conn).unwrap();
        let qh = event_queue.handle();

        let source = WaylandSource::new(conn.clone(), event_queue);
        handle
            .clone()
            .insert_source(source, |_, queue, data| {
                queue.dispatch_pending(&mut data.lock().sm)
            })
            .expect("could not insert");

        let (tx, rx) = channel();

        handle
            .insert_source(rx, |e, _, a| {
                let a = &mut a.lock().sm;

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
                                RecreateRequest::Layershell => {
                                    todo!();
                                }
                                RecreateRequest::Popup(positioner, parent) => {
                                    let popup = match parent {
                                        PopupParent::XdgSurface(parent) => {
                                            let popup = ToolkitPopup::from_surface(
                                                Some(&parent),
                                                &positioner.build(),
                                                &a.qh,
                                                surface,
                                                &a.xdg_shell,
                                            )
                                            .expect("could not create toolkit popup");
                                            popup.wl_surface().commit();
                                            popup
                                        }
                                        PopupParent::LayerSurface(parent) => {
                                            let popup = ToolkitPopup::from_surface(
                                                None,
                                                &positioner.build(),
                                                &a.qh,
                                                surface,
                                                &a.xdg_shell,
                                            )
                                            .expect("could not create toolkit popup");
                                            parent.get_popup(&popup.xdg_popup());
                                            popup.wl_surface().commit();
                                            popup
                                        }
                                    };

                                    RecreateResponse::Popup(popup)
                                }
                            };

                            a.surface_id_to_id.insert(surface_id.clone(), id);
                            let state = a.state_mut(&surface_id);
                            let mut state = state.lock().unwrap();
                            state.map(response);
                        }
                        Request::Unmap { id } => {
                            a.surface_id_to_id.remove(&id);
                        }
                        Request::PopupGrab { popup, serial } => {
                            popup
                                .xdg_popup()
                                .grab(&a.seat_state.seats().next().unwrap(), serial as u32);
                        }
                    }
                }
            })
            .unwrap();

        Ok(Self {
            active_keyboard: None,

            backend,

            keyboard: None,
            pointer: None,

            compositor_state: CompositorState::bind(&globals, &qh)?,
            registry_state: RegistryState::new(&globals),
            seat_state: SeatState::new(&globals, &qh),
            output_state: OutputState::new(&globals, &qh),

            cursor_shape_manager: CursorShapeManager::bind(&globals, &qh)?,
            last_enter_serial: 0,

            xdg_shell: XdgShell::bind(&globals, &qh)?,
            layershell: LayerShell::bind(&globals, &qh)?,

            qh,
            tx,

            counter: 0,

            states: HashMap::new(),
            surface_id_to_id: HashMap::new(),
        })
    }

    pub fn state_mut(&mut self, id: &ObjectId) -> Arc<Mutex<dyn State>> {
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

    pub fn get_positioner(&mut self) -> anyhow::Result<Positioner> {
        let xdg_positioner = XdgPositioner::new(&self.xdg_shell)?;
        Ok(Positioner::new(xdg_positioner))
    }

    pub fn prepare_popup(&mut self) -> anyhow::Result<Arc<Mutex<Popup>>> {
        let id = self.counter.clone();
        let dummy = self.compositor_state.create_surface(&self.qh);
        let popup = Popup::new(id, self.tx.clone(), self.backend, &dummy)?;
        let popup = Arc::new(Mutex::new(popup));

        self.states.insert(id, popup.clone());
        self.counter += 1;

        Ok(popup)
    }

    pub fn create_window(
        &mut self,
        width: usize,
        height: usize,
    ) -> anyhow::Result<Arc<Mutex<Window>>> {
        let surface = self.compositor_state.create_surface(&self.qh);
        let surface_id = surface.id();
        let xdg_window = self
            .xdg_shell
            .create_window(surface, WindowDecorations::None, &self.qh);

        xdg_window.set_min_size(Some((width as u32, height as u32)));
        xdg_window.commit();

        let tx = self.tx.clone();
        let id = self.counter.clone();

        let window = Window::new(id, tx, self.backend, xdg_window)?;
        let window = Arc::new(Mutex::new(window));

        self.states.insert(id, window.clone());
        self.surface_id_to_id.insert(surface_id, id);
        self.counter += 1;

        Ok(window)
    }

    pub fn create_layer(
        &mut self,
        layer: smithay_client_toolkit::shell::wlr_layer::Layer,
        namespace: String,
    ) -> anyhow::Result<Arc<Mutex<LayerSurface>>> {
        let surface = self.compositor_state.create_surface(&self.qh);
        let surface_id = surface.id();

        let wlr_layer_surface =
            self.layershell
                .create_layer_surface(&self.qh, surface, layer, Some(namespace), None);

        let tx = self.tx.clone();
        let id = self.counter.clone();

        let layer_surface = LayerSurface::new(id, tx, self.backend, wlr_layer_surface)?;
        let layer_surface = Arc::new(Mutex::new(layer_surface));

        self.states.insert(id, layer_surface.clone());
        self.surface_id_to_id.insert(surface_id, id);
        self.counter += 1;

        Ok(layer_surface)
    }
}

impl<'a> CompositorHandler for SurfaceManager<'a>
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
        println!("requesting frame");
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

impl<'a> OutputHandler for SurfaceManager<'a>
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

impl<'a> WindowHandler for SurfaceManager<'a>
where
    'a: 'static,
{
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &XdgWindow) {
        if !self.has_state(&window.wl_surface().id()) {
            return;
        }

        let id = {
            let state = self.state_mut(&window.wl_surface().id());
            let mut state = state.lock().unwrap();
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
        let mut state = state.lock().unwrap();

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

impl<'a> SeatHandler for SurfaceManager<'a>
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
            // FIXME: recreate repeats with a custom implementation which allows LoopHandle<Application>
            //let keyboard = self
            //    .seat_state
            //    .get_keyboard_with_repeat(
            //        qh,
            //        &seat,
            //        None,
            //        self.loop_handle.clone(),
            //        Box::new(move |app, _, event| match app.active_keyboard.clone() {
            //            Some(id) => {
            //                if !app.has_state(&id) {
            //                    return;
            //                }
            //                let state = app.state_mut(&id);
            //                let mut state = state.lock().unwrap();
            //                state.key_event(&event, KeyState::Pressed);
            //            }
            //            None => {}
            //        }),
            //    )
            //    .expect("Failed to create keyboard");
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
        _capability: Capability,
    ) {
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl<'a> KeyboardHandler for SurfaceManager<'a>
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
        println!("keyboard leave");
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
                let mut state = state.lock().unwrap();
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

impl<'a> PointerHandler for SurfaceManager<'a>
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
            let mut state = state.lock().unwrap();

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

impl<'a> PopupHandler for SurfaceManager<'a>
where
    'a: 'static,
{
    fn done(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, popup: &ToolkitPopup) {
        if !self.has_state(&popup.wl_surface().id()) {
            return;
        }

        let id = {
            let state = self.state_mut(&popup.wl_surface().id());
            let mut state = state.lock().unwrap();
            state.exit();
            state.id()
        };

        self.surface_id_to_id.remove(&popup.wl_surface().id());
        self.states.remove(&id);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        popup: &ToolkitPopup,
        config: smithay_client_toolkit::shell::xdg::popup::PopupConfigure,
    ) {
        if !self.has_state(&popup.wl_surface().id()) {
            return;
        }
        let state = self.state_mut(&popup.wl_surface().id());
        let mut state = state.lock().unwrap();

        state.resize(config.width as usize, config.height as usize, 0, 0);
    }
}

impl<'a> LayerShellHandler for SurfaceManager<'a>
where
    'a: 'static,
{
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &WlrLayerSurface) {}

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &WlrLayerSurface,
        configure: smithay_client_toolkit::shell::wlr_layer::LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if !self.has_state(&layer.wl_surface().id()) {
            return;
        }

        let state = self.state_mut(&layer.wl_surface().id());
        let mut state = state.lock().unwrap();

        let width = configure.new_size.0;
        let height = configure.new_size.1;

        state.resize(width as usize, height as usize, 0, 0);
    }
}

delegate_compositor!(@<'a: 'static> SurfaceManager<'a>);
delegate_output!(@<'a: 'static>SurfaceManager<'a>);

delegate_seat!(@<'a: 'static>SurfaceManager<'a>);
delegate_keyboard!(@<'a: 'static>SurfaceManager<'a>);
delegate_pointer!(@<'a: 'static>SurfaceManager<'a>);

delegate_xdg_shell!(@<'a: 'static>SurfaceManager<'a>);
delegate_xdg_window!(@<'a: 'static>SurfaceManager<'a>);
delegate_xdg_popup!(@<'a: 'static>SurfaceManager<'a>);

delegate_layer!(@<'a: 'static>SurfaceManager<'a>);

delegate_registry!(@<'a: 'static>SurfaceManager<'a>);

impl<'a> ProvidesRegistryState for SurfaceManager<'a>
where
    'a: 'static,
{
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers![OutputState, SeatState,];
}
