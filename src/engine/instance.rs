use std::{
    ffi::{CStr, CString},
    time::Duration,
};

use crate::backend::Backend;
use bindings::{
    FlutterEngineRunInitialized, FlutterEngineSendPlatformMessage, FlutterPointerPhase_kDown,
    FlutterProjectArgs, FlutterRendererConfig,
};

use crate::shell::{
    channel,
    timer::{self, TimeoutAction},
    LoopHandle,
};

use super::{
    builtin::{self, Decorations, Textinput},
    stream, CsdMessage, Plugin, PluginMessage, PointerButtons, Shell,
};

struct Userdata {
    backend: Backend,

    plugins: Vec<Box<dyn super::Plugin>>,
    task_runner: crate::tasks::TaskRunner,
}

pub struct Instance {
    userdata: Box<Userdata>,

    inner: bindings::FlutterEngine,
}

unsafe extern "C" fn renderer_clear_current(data: *mut std::ffi::c_void) -> bool {
    let ustadata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    ustadata.backend.clear_current().is_ok()
}

unsafe extern "C" fn renderer_make_current(data: *mut std::ffi::c_void) -> bool {
    let userdata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    userdata.backend.make_current(0).is_ok()
}

unsafe extern "C" fn renderer_present(data: *mut std::ffi::c_void) -> bool {
    let ustadata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    ustadata.backend.swap_buffers(0).is_ok()
}

unsafe extern "C" fn renderer_fbo_callback(_data: *mut std::ffi::c_void) -> u32 {
    0
}

unsafe extern "C" fn platform_message_callback(
    message: *const bindings::FlutterPlatformMessage,
    data: *mut std::ffi::c_void,
) {
    let userdata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };

    let message = unsafe { &*message };
    let channel = unsafe { CStr::from_ptr(message.channel) }
        .to_str()
        .to_owned()
        .unwrap();

    let payload = {
        let slice = std::slice::from_raw_parts(message.message, message.message_size);
        let mut buffer = Vec::with_capacity(message.message_size);
        buffer.extend_from_slice(slice);
        buffer
    };

    for plugin in &mut userdata.plugins {
        if plugin.on() == channel {
            match plugin.handle(payload.clone()) {
                Ok(_) => {}
                Err(e) => {
                    log::error!(
                        "plugin for: {} failed to handle message because: {}",
                        channel,
                        e.to_string()
                    );
                }
            }
        }
    }
}

unsafe extern "C" fn renderer_proc_resolver(
    data: *mut std::ffi::c_void,
    name: *const i8,
) -> *mut std::ffi::c_void {
    let ustadata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    let cstr = std::ffi::CStr::from_ptr(name);
    ustadata
        .backend
        .get_proc_address(cstr.to_str().unwrap())
        .expect("could not get proc address") as *mut std::ffi::c_void
}

pub struct Config {
    pub assets: String,
    pub icu_data: String,
}

impl Instance {
    pub fn new(backend: Backend, instance_config: Config) -> anyhow::Result<Self> {
        let config = bindings::FlutterRendererConfig {
            type_: bindings::FlutterRendererType_kOpenGL,
            __bindgen_anon_1: bindings::FlutterRendererConfig__bindgen_ty_1 {
                open_gl: bindings::FlutterOpenGLRendererConfig {
                    struct_size: std::mem::size_of::<bindings::FlutterOpenGLRendererConfig>(),
                    clear_current: Some(renderer_clear_current),
                    fbo_callback: Some(renderer_fbo_callback),
                    fbo_reset_after_present: false,
                    fbo_with_frame_info_callback: None,
                    gl_external_texture_frame_callback: None,
                    gl_proc_resolver: Some(renderer_proc_resolver),
                    make_current: Some(renderer_make_current),
                    make_resource_current: None,
                    populate_existing_damage: None,
                    present_with_info: None,
                    present: Some(renderer_present),
                    surface_transformation: None,
                },
            },
        };

        let userdata = Userdata {
            backend,
            plugins: vec![],
            task_runner: crate::tasks::TaskRunner::new()?,
        };

        let mut instance = Self {
            userdata: Box::new(userdata),
            inner: std::ptr::null_mut(),
        };

        instance.with(Box::new(builtin::Textinput::new()));
        instance.with(Box::new(builtin::Mousecursor::new()));
        // instance.with(Box::new(builtin::Decorations::new()));

        let mut task_runner_description: bindings::FlutterTaskRunnerDescription =
            unsafe { std::mem::zeroed() };
        task_runner_description.struct_size =
            std::mem::size_of::<bindings::FlutterTaskRunnerDescription>();
        task_runner_description.post_task_callback = Some(crate::tasks::TaskRunner::post_task);
        task_runner_description.runs_task_on_current_thread_callback =
            Some(crate::tasks::TaskRunner::runs_on_thread);
        task_runner_description.user_data =
            &mut instance.userdata.task_runner as *mut _ as *mut std::ffi::c_void;

        let mut custom_task_runners: bindings::FlutterCustomTaskRunners =
            unsafe { std::mem::zeroed() };

        custom_task_runners.struct_size = std::mem::size_of::<bindings::FlutterCustomTaskRunners>();
        custom_task_runners.platform_task_runner = &task_runner_description as *const _ as _;

        let mut args: bindings::FlutterProjectArgs = unsafe { std::mem::zeroed() };

        let assets_path = CString::new(instance_config.assets)?;
        let icu_data_path = CString::new(instance_config.icu_data)?;

        args.struct_size = std::mem::size_of::<bindings::FlutterProjectArgs>();
        args.assets_path = assets_path.as_ptr() as *const i8;
        args.icu_data_path = icu_data_path.as_ptr() as *const i8;

        args.platform_message_callback = Some(platform_message_callback);
        args.custom_task_runners = &custom_task_runners as *const _ as _;

        unsafe {
            bindings::FlutterEngineInitialize(
                bindings::FLUTTER_ENGINE_VERSION as usize,
                &config as *const FlutterRendererConfig,
                &args as *const FlutterProjectArgs,
                &*instance.userdata as *const _ as _,
                &mut instance.inner,
            )
        };

        Ok(instance)
    }

    pub fn with(&mut self, plugin: Box<dyn Plugin>) {
        for other in &self.userdata.plugins {
            if plugin.on() == other.on() {
                log::error!("plugin for: {} is already installed", plugin.on());
                todo!("do something more gracefull here");
            }
        }
        self.userdata.plugins.push(plugin);
    }

    pub fn run<'a, T: Shell>(
        &mut self,
        handle: LoopHandle<'a, T>,
        stream: channel::Channel<stream::Event>,
    ) -> anyhow::Result<()> {
        let result = unsafe { FlutterEngineRunInitialized(self.inner) };
        if result != bindings::FlutterEngineResult_kSuccess {
            return Err(super::InstanceError::RunFailed.into());
        }

        let (tx, plugin_messages) = channel::channel();

        for plugin in &mut self.userdata.plugins {
            plugin.init(tx.clone(), super::ShellCapabilities::all())?;
        }

        let inner = self.inner.clone();
        let mut last_phase = 0;

        handle
            .insert_source(stream, move |e, _, s| {
                if let channel::Event::Msg(event) = e {
                    match event {
                        stream::Event::Resized { width, height } => {
                            let resize_event = bindings::FlutterWindowMetricsEvent {
                                struct_size: std::mem::size_of::<bindings::FlutterWindowMetricsEvent>(),
                                width: width as usize,
                                height: height as usize,
                                pixel_ratio: 1.0,
                                top: 0,
                                left: 0,
                                physical_view_inset_top: 0.0,
                                physical_view_inset_left: 0.0,
                                physical_view_inset_right: 0.0,
                                physical_view_inset_bottom: 0.0,
                            };

                            unsafe {
                                bindings::FlutterEngineSendWindowMetricsEvent(inner, &resize_event as *const _);
                            };
                        }
                        stream::Event::Close => {
                            unsafe {
                                bindings::FlutterEngineShutdown(inner);
                            }
                        }
                        stream::Event::PointerEnter { x, y } => {
                            unsafe {
                                let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
                                event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
                                event.phase = bindings::FlutterPointerPhase_kAdd;
                                event.x = x;
                                event.y = y;
                                event.device = 69;
                                event.device_kind = bindings::FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse;
                                bindings::FlutterEngineSendPointerEvent(inner, &event as *const bindings::FlutterPointerEvent, 1);
                            }
                        }
                        stream::Event::PointerLeave { .. } => {
                            unsafe {
                                let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
                                event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
                                event.phase = bindings::FlutterPointerPhase_kRemove;
                                event.device = 69;
                                event.device_kind = bindings::FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse;
                                bindings::FlutterEngineSendPointerEvent(inner, &event as *const bindings::FlutterPointerEvent, 1);
                            }
                        }
                        stream::Event::PointerMotion { x, y, time, serial }=> {
                            unsafe {
                                let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
                                event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
                                event.phase = bindings::FlutterPointerPhase_kHover;
                                if last_phase == FlutterPointerPhase_kDown {
                                    event.phase = bindings::FlutterPointerPhase_kMove;
                                }

                                if event.phase == bindings::FlutterPointerPhase_kMove {
                                    match s.instance_mut().userdata.plugins.iter_mut().find(|p| p.on() == "isabel/decorations") {
                                        Some(p) => {
                                            let decorations = p.downcast_mut::<Decorations>().unwrap();
                                            decorations.move_(serial);
                                        },
                                        None => {},
                                    }
                                }

                                event.x = x;
                                event.y = y;
                                event.timestamp = time as usize;

                                event.device = 69;
                                event.device_kind = bindings::FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse;
                                bindings::FlutterEngineSendPointerEvent(inner, &event as *const bindings::FlutterPointerEvent, 1);
                            }
                        }
                        stream::Event::PointerButton { x, y, button, time, pressed, .. } => {
                            unsafe {
                                let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
                                event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
                                event.phase = if pressed { bindings::FlutterPointerPhase_kDown } else { bindings::FlutterPointerPhase_kUp};
                                event.x = x;
                                event.y = y;
                                event.timestamp = time as usize;
                                event.buttons = match button {
                                    PointerButtons::Primary =>
                                        bindings::FlutterPointerMouseButtons_kFlutterPointerButtonMousePrimary,
                                    PointerButtons::Secondary => bindings::FlutterPointerMouseButtons_kFlutterPointerButtonMouseSecondary,
                                    PointerButtons::Back => bindings::FlutterPointerMouseButtons_kFlutterPointerButtonMouseBack,
                                    PointerButtons::Middle => bindings::FlutterPointerMouseButtons_kFlutterPointerButtonMouseMiddle,
                                } as i64;

                                event.device = 69;
                                event.device_kind = bindings::FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse;
                                bindings::FlutterEngineSendPointerEvent(inner, &event as *const bindings::FlutterPointerEvent, 1);

                                println!("phase: {:?}", event.phase);

                                last_phase = event.phase;
                            }
                        }
                        stream::Event::PointerScroll => {}
                        stream::Event::Keypress { symbol, pressed } => {
                            let plugin = s.instance_mut().userdata.plugins.iter_mut().find(|p| p.on() == "flutter/textinput").unwrap();
                            let textinput = plugin.downcast_mut::<Textinput>().unwrap();
                            textinput.handle_key_event(symbol, pressed);
                        }
                    }
                }
            })
            .expect("could not insert channel");

        handle
            .insert_source(
                timer::Timer::from_duration(Duration::from_nanos(200)),
                move |_, _, s| {
                    let instance = s.instance_mut();
                    instance.userdata.task_runner.process(inner);
                    TimeoutAction::ToDuration(Duration::from_nanos(200))
                },
            )
            .expect("could not insert timer");

        handle
            .insert_source(plugin_messages, move |e, _, s| {
                if let channel::Event::Msg(event) = e {
                    match event {
                        PluginMessage::SetCursor { icon } => {
                            s.set_cursor_icon(icon).unwrap();
                        }
                        PluginMessage::PlatformMessage { payload, channel } => unsafe {
                            let channel_cstring = CString::new(channel).unwrap();

                            let mut message =
                                std::mem::zeroed::<bindings::FlutterPlatformMessage>();
                            message.struct_size =
                                std::mem::size_of::<bindings::FlutterPlatformMessage>();
                            message.channel = channel_cstring.as_ptr();
                            message.message = payload.as_ptr();
                            message.message_size = payload.len();

                            FlutterEngineSendPlatformMessage(
                                inner,
                                &message as *const bindings::FlutterPlatformMessage,
                            );
                        },

                        PluginMessage::Csd(csd) => match csd {
                            CsdMessage::Move(serial) => {
                                s.initiate_move(serial);
                            }
                        },
                    }
                }
            })
            .expect("could not insert channel");

        Ok(())
    }
}
