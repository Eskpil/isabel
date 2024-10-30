use std::{
    ffi::{CStr, CString},
    sync::{Arc, Mutex},
};

use smithay_client_toolkit::reexports::calloop::{
    channel::{channel, Channel, ChannelError, Event, Sender},
    EventSource,
};
use thiserror::Error;

use crate::{
    backend::{Backend, MAIN_SURFACE},
    tasks::TaskRunner,
};

use super::PointerButtons;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum EngineError {
    #[error("Running the flutter engine failed")]
    RunFailed,

    #[error("Unable to get the engine proc table")]
    ProcTableFailed,

    #[error("Unable to create aot data")]
    CreateAotData,

    #[error("Resizing the engine failed")]
    ResizeFailed,

    #[error("Pointer event failed to execute")]
    PointerEvent,

    #[error("Failed to shutdown the engine")]
    ShutdownFailed,
}

pub enum EngineEvent {
    PlatformMessage { channel: String, data: Vec<u8> },
}

struct Userdata {
    backend: Arc<Mutex<Backend>>,
    tx: Sender<EngineEvent>,
}

pub struct Engine {
    inner: bindings::FlutterEngine,
    proc_table: bindings::FlutterEngineProcTable,
    args: bindings::FlutterProjectArgs,

    assets_path: String,
    icu_data_path: String,

    userdata: Box<Userdata>,

    platform_task_runner: Option<TaskRunner>,

    last_phase: bindings::FlutterPointerPhase,
}

pub struct EngineSource {
    channel: Channel<EngineEvent>,
}

unsafe extern "C" fn renderer_clear_current(data: *mut std::ffi::c_void) -> bool {
    let userdata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    let backend = userdata.backend.lock().unwrap();
    if backend.has(&MAIN_SURFACE) {
        backend.clear_current().is_ok()
    } else {
        // Trick the flutter engine into thinking everything is ok with rendereing. Although it is not clearing any surfaces.
        // This is part of how we are able to hide the surface its drawing to but still keep the instance running.
        true
    }
}

unsafe extern "C" fn renderer_make_current(data: *mut std::ffi::c_void) -> bool {
    let userdata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    let backend = userdata.backend.lock().unwrap();
    if backend.has(&MAIN_SURFACE) {
        backend.make_current(&MAIN_SURFACE).is_ok()
    } else {
        true
    }
}

unsafe extern "C" fn renderer_present(data: *mut std::ffi::c_void) -> bool {
    let userdata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    let backend = userdata.backend.lock().unwrap();
    if backend.has(&MAIN_SURFACE) {
        backend.swap_buffers(&MAIN_SURFACE).is_ok()
    } else {
        true
    }
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
        .unwrap()
        .to_owned();

    let data = {
        Vec::from_raw_parts(
            message.message as _,
            message.message_size,
            message.message_size,
        )
    };

    userdata
        .tx
        .send(EngineEvent::PlatformMessage { channel, data })
        .expect("could not publish engine event");
}

unsafe extern "C" fn renderer_proc_resolver(
    data: *mut std::ffi::c_void,
    name: *const i8,
) -> *mut std::ffi::c_void {
    let userdata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    let cstr = std::ffi::CStr::from_ptr(name);
    userdata
        .backend
        .lock()
        .unwrap()
        .get_proc_address(cstr.to_str().unwrap())
        .expect("could not get proc address") as *mut std::ffi::c_void
}

impl Engine {
    pub fn new(
        backend: Arc<Mutex<Backend>>,
        assets_path: String,
        icu_data_path: String,
    ) -> anyhow::Result<(Self, EngineSource)> {
        let mut proc_table: bindings::FlutterEngineProcTable = unsafe { std::mem::zeroed() };
        proc_table.struct_size = std::mem::size_of::<bindings::FlutterEngineProcTable>();
        unsafe {
            let result = bindings::FlutterEngineGetProcAddresses(&mut proc_table as *mut _);
            if result != bindings::FlutterEngineResult_kSuccess {
                return Err(EngineError::ProcTableFailed.into());
            }
        }

        let (tx, rx) = channel();

        let userdata = Box::new(Userdata { backend, tx });

        let mut args: bindings::FlutterProjectArgs = unsafe { std::mem::zeroed() };

        args.struct_size = std::mem::size_of::<bindings::FlutterProjectArgs>();
        args.platform_message_callback = Some(platform_message_callback);

        Ok((
            Self {
                userdata,
                proc_table,
                args,

                platform_task_runner: None,

                assets_path,
                icu_data_path,

                inner: std::ptr::null_mut(),
                last_phase: 0,
            },
            EngineSource { channel: rx },
        ))
    }

    pub fn exit(&mut self) -> anyhow::Result<()> {
        let result = unsafe { bindings::FlutterEngineShutdown(self.inner()) };
        if result != bindings::FlutterEngineResult_kSuccess {
            return Err(EngineError::ShutdownFailed.into());
        }

        Ok(())
    }

    pub fn inner(&self) -> bindings::FlutterEngine {
        self.inner
    }

    pub fn publish(&mut self, channel: String, payload: Vec<u8>) {
        unsafe {
            let channel_cstring = CString::new(channel).unwrap();

            let mut message = std::mem::zeroed::<bindings::FlutterPlatformMessage>();
            message.struct_size = std::mem::size_of::<bindings::FlutterPlatformMessage>();
            message.channel = channel_cstring.as_ptr();
            message.message = payload.as_ptr();
            message.message_size = payload.len();

            bindings::FlutterEngineSendPlatformMessage(
                self.inner,
                &message as *const bindings::FlutterPlatformMessage,
            );
        }
    }

    pub fn add_platform_task_runner(&mut self, runner: TaskRunner) {
        self.platform_task_runner = Some(runner);
    }

    pub fn runs_aot(&self) -> bool {
        if unsafe { self.proc_table.RunsAOTCompiledDartCode.unwrap()() } {
            true
        } else {
            false
        }
    }

    pub fn create_aot_data(&mut self, path: String) -> anyhow::Result<()> {
        let mut source = unsafe { std::mem::zeroed::<bindings::FlutterEngineAOTDataSource>() };
        source.type_ =
            bindings::FlutterEngineAOTDataSourceType_kFlutterEngineAOTDataSourceTypeElfPath;

        let aot_path = CString::new(path)?;
        source.__bindgen_anon_1.elf_path = aot_path.as_ptr() as *const i8;

        let result = unsafe {
            self.proc_table.CreateAOTData.unwrap()(
                &source as *const _,
                &mut self.args.aot_data as *mut _,
            )
        };

        if result != bindings::FlutterEngineResult_kSuccess {
            return Err(EngineError::CreateAotData.into());
        }

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut args = self.args.clone();

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

        let assets_path = CString::new(self.assets_path.clone())?;
        let icu_data_path = CString::new(self.icu_data_path.clone())?;

        if let Some(task_runner) = &mut self.platform_task_runner {
            let mut task_runner_description: bindings::FlutterTaskRunnerDescription =
                unsafe { std::mem::zeroed() };
            task_runner_description.struct_size =
                std::mem::size_of::<bindings::FlutterTaskRunnerDescription>();
            task_runner_description.post_task_callback = Some(crate::tasks::TaskRunner::post_task);
            task_runner_description.runs_task_on_current_thread_callback =
                Some(crate::tasks::TaskRunner::runs_on_thread);
            task_runner_description.user_data = task_runner as *mut _ as *mut std::ffi::c_void;

            let mut custom_task_runners: bindings::FlutterCustomTaskRunners =
                unsafe { std::mem::zeroed() };

            custom_task_runners.struct_size =
                std::mem::size_of::<bindings::FlutterCustomTaskRunners>();
            custom_task_runners.platform_task_runner = &task_runner_description as *const _ as _;

            args.custom_task_runners = &custom_task_runners as *const _ as _;
        }

        args.assets_path = assets_path.as_ptr();
        args.icu_data_path = icu_data_path.as_ptr();

        let result = unsafe {
            bindings::FlutterEngineRun(
                bindings::FLUTTER_ENGINE_VERSION as usize,
                &config as _,
                &args as _,
                &*self.userdata as *const _ as _,
                &mut self.inner,
            )
        };
        if result != bindings::FlutterEngineResult_kSuccess {
            return Err(EngineError::RunFailed.into());
        }

        if let Some(task_runner) = &mut self.platform_task_runner {
            task_runner.set_engine(self.inner);
        }

        Ok(())
    }

    pub fn resize(&mut self, width: usize, height: usize) -> anyhow::Result<()> {
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

        let result = unsafe {
            bindings::FlutterEngineSendWindowMetricsEvent(self.inner, &resize_event as *const _)
        };

        if result != bindings::FlutterEngineResult_kSuccess {
            return Err(EngineError::ResizeFailed.into());
        }

        Ok(())
    }

    pub fn shutdown(&mut self) -> anyhow::Result<()> {
        unsafe {
            bindings::FlutterEngineShutdown(self.inner);
        };

        Ok(())
    }

    pub fn pointer_enter(&mut self, x: f64, y: f64) -> anyhow::Result<()> {
        let result = unsafe {
            let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
            event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
            event.phase = bindings::FlutterPointerPhase_kAdd;
            event.x = x;
            event.y = y;
            event.device = 69;
            event.device_kind = bindings::FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse;

            bindings::FlutterEngineSendPointerEvent(
                self.inner,
                &event as *const bindings::FlutterPointerEvent,
                1,
            )
        };

        if result != bindings::FlutterEngineResult_kSuccess {
            return Err(EngineError::PointerEvent.into());
        }

        Ok(())
    }

    pub fn pointer_leave(&mut self) -> anyhow::Result<()> {
        let result = unsafe {
            let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
            event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
            event.phase = bindings::FlutterPointerPhase_kRemove;
            event.device = 69;
            event.device_kind = bindings::FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse;
            bindings::FlutterEngineSendPointerEvent(
                self.inner,
                &event as *const bindings::FlutterPointerEvent,
                1,
            )
        };

        if result != bindings::FlutterEngineResult_kSuccess {
            return Err(EngineError::PointerEvent.into());
        }

        Ok(())
    }

    pub fn pointer_motion(&mut self, x: f64, y: f64, time: usize) -> anyhow::Result<()> {
        let result = unsafe {
            let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
            event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
            event.phase = bindings::FlutterPointerPhase_kHover;
            if self.last_phase == bindings::FlutterPointerPhase_kDown {
                event.phase = bindings::FlutterPointerPhase_kMove;
            }

            event.x = x;
            event.y = y;
            event.timestamp = time as usize;

            event.device = 69;
            event.device_kind = bindings::FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse;
            bindings::FlutterEngineSendPointerEvent(
                self.inner,
                &event as *const bindings::FlutterPointerEvent,
                1,
            )
        };

        if result != bindings::FlutterEngineResult_kSuccess {
            return Err(EngineError::PointerEvent.into());
        }

        Ok(())
    }

    pub fn pointer_button(
        &mut self,
        x: f64,
        y: f64,
        time: u32,
        button: PointerButtons,
        pressed: bool,
    ) -> anyhow::Result<()> {
        let result = unsafe {
            let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
            event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
            event.phase = if pressed {
                bindings::FlutterPointerPhase_kDown
            } else {
                bindings::FlutterPointerPhase_kUp
            };
            event.x = x;
            event.y = y;
            event.timestamp = time as usize;
            event.buttons = match button {
                PointerButtons::Primary => {
                    bindings::FlutterPointerMouseButtons_kFlutterPointerButtonMousePrimary
                }
                PointerButtons::Secondary => {
                    bindings::FlutterPointerMouseButtons_kFlutterPointerButtonMouseSecondary
                }
                PointerButtons::Back => {
                    bindings::FlutterPointerMouseButtons_kFlutterPointerButtonMouseBack
                }
                PointerButtons::Middle => {
                    bindings::FlutterPointerMouseButtons_kFlutterPointerButtonMouseMiddle
                }
            } as i64;

            event.device = 69;
            event.device_kind = bindings::FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse;

            self.last_phase = event.phase;

            bindings::FlutterEngineSendPointerEvent(
                self.inner,
                &event as *const bindings::FlutterPointerEvent,
                1,
            )
        };

        if result != bindings::FlutterEngineResult_kSuccess {
            return Err(EngineError::PointerEvent.into());
        }

        Ok(())
    }
}

impl EventSource for EngineSource {
    type Event = EngineEvent;
    type Metadata = ();
    type Ret = ();
    type Error = ChannelError;

    fn register(
        &mut self,
        poll: &mut smithay_client_toolkit::reexports::calloop::Poll,
        token_factory: &mut smithay_client_toolkit::reexports::calloop::TokenFactory,
    ) -> smithay_client_toolkit::reexports::calloop::Result<()> {
        self.channel.register(poll, token_factory)
    }

    fn reregister(
        &mut self,
        poll: &mut smithay_client_toolkit::reexports::calloop::Poll,
        token_factory: &mut smithay_client_toolkit::reexports::calloop::TokenFactory,
    ) -> smithay_client_toolkit::reexports::calloop::Result<()> {
        self.channel.register(poll, token_factory)
    }

    fn unregister(
        &mut self,
        poll: &mut smithay_client_toolkit::reexports::calloop::Poll,
    ) -> smithay_client_toolkit::reexports::calloop::Result<()> {
        self.channel.unregister(poll)
    }

    fn process_events<F>(
        &mut self,
        readiness: smithay_client_toolkit::reexports::calloop::Readiness,
        token: smithay_client_toolkit::reexports::calloop::Token,
        mut callback: F,
    ) -> Result<smithay_client_toolkit::reexports::calloop::PostAction, Self::Error>
    where
        F: FnMut(Self::Event, &mut Self::Metadata) -> Self::Ret,
    {
        let action = self
            .channel
            .process_events(readiness, token, |e, &mut ()| match e {
                Event::Msg(m) => {
                    callback(m, &mut ());
                }
                Event::Closed => {
                    // TODO: Handle this
                }
            })?;

        Ok(action)
    }
}
