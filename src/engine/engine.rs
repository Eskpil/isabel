use std::ffi::{CStr, CString};

use smithay_client_toolkit::reexports::calloop::{
    channel::{channel, Channel, ChannelError, Event, Sender},
    EventSource,
};
use thiserror::Error;

use crate::{
    backend::{Backend, Surface},
    sm::KeyState,
    tasks::{TaskRunner, TaskRunnerClient},
};

use super::{
    keys::{translate_logical_key, translate_physical_key},
    PointerButtons,
};

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

    #[error("Failed to send key event")]
    KeyEvent,
}

pub enum EngineEvent {
    PlatformMessage { channel: String, data: Vec<u8> },
}

struct Userdata {
    surface: Surface,
    tx: Sender<EngineEvent>,
}

pub struct Engine {
    inner: bindings::FlutterEngine,
    proc_table: bindings::FlutterEngineProcTable,
    args: bindings::FlutterProjectArgs,

    assets_path: CString,
    icu_data_path: CString,
    pub entry: Option<CString>,

    userdata: Box<Userdata>,

    platform_task_runner: TaskRunnerClient,

    last_phase: bindings::FlutterPointerPhase,
}

pub struct EngineSource {
    channel: Channel<EngineEvent>,
}

unsafe extern "C" fn renderer_clear_current(data: *mut std::ffi::c_void) -> bool {
    let userdata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    userdata.surface.clear_current().is_ok()
}

unsafe extern "C" fn renderer_make_current(data: *mut std::ffi::c_void) -> bool {
    let userdata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    userdata.surface.make_current().is_ok()
}

unsafe extern "C" fn renderer_present(data: *mut std::ffi::c_void) -> bool {
    let userdata: &mut Userdata = unsafe { &mut *(data as *mut Userdata) };
    userdata.surface.swap_buffers().is_ok()
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

    if userdata.surface.present() {
        let cstr = std::ffi::CStr::from_ptr(name).to_str().unwrap();
        Backend::get_proc_address(cstr).expect("could not resolve proc address")
            as *mut std::ffi::c_void
    } else {
        std::ptr::null_mut()
    }
}

unsafe extern "C" fn keydata_callback(handled: bool, userdata: *mut std::ffi::c_void) {
    println!("hello {handled}");
}

impl Engine {
    pub fn new(
        surface: Surface,
        assets_path: String,
        icu_data_path: String,
        task_runner: TaskRunner,
        entry: Option<String>,
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

        let userdata = Box::new(Userdata { surface, tx });

        let mut args: bindings::FlutterProjectArgs = unsafe { std::mem::zeroed() };

        args.struct_size = std::mem::size_of::<bindings::FlutterProjectArgs>();
        args.platform_message_callback = Some(platform_message_callback);

        Ok((
            Self {
                userdata,
                proc_table,
                args,

                platform_task_runner: task_runner.fork(),

                entry: entry.map_or(None, |s| Some(CString::new(s).unwrap())),

                assets_path: CString::new(assets_path)?,
                icu_data_path: CString::new(icu_data_path)?,

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

        let mut task_runner_description: bindings::FlutterTaskRunnerDescription =
            unsafe { std::mem::zeroed() };
        task_runner_description.struct_size =
            std::mem::size_of::<bindings::FlutterTaskRunnerDescription>();
        task_runner_description.post_task_callback = Some(TaskRunnerClient::post_task);

        task_runner_description.runs_task_on_current_thread_callback =
            Some(TaskRunnerClient::runs_on_thread);
        task_runner_description.user_data =
            &mut self.platform_task_runner as *mut _ as *mut std::ffi::c_void;
        task_runner_description.identifier = 0;

        let mut custom_task_runners: bindings::FlutterCustomTaskRunners =
            unsafe { std::mem::zeroed() };

        custom_task_runners.struct_size = std::mem::size_of::<bindings::FlutterCustomTaskRunners>();
        custom_task_runners.platform_task_runner = &task_runner_description as *const _ as _;

        args.custom_task_runners = &custom_task_runners as *const _ as _;

        args.assets_path = self.assets_path.as_ptr();
        args.icu_data_path = self.icu_data_path.as_ptr();

        let disable_service_auth_codes = CString::new("--disable-service-auth-codes").unwrap();
        let disable_observatory = CString::new("--disable-observatory").unwrap();
        let dart_vm_args = vec![
            disable_service_auth_codes.as_ptr(),
            disable_observatory.as_ptr(),
        ];

        if self.entry.is_some() {
            args.dart_entrypoint_argc = dart_vm_args.len() as i32;
            args.dart_entrypoint_argv = dart_vm_args.as_ptr();
        }

        if let Some(entry) = &self.entry {
            args.custom_dart_entrypoint = entry.as_ptr();
        }

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

        self.platform_task_runner.set_engine(self.inner());

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

    pub fn pointer_motion(&mut self, x: f64, y: f64, time: u64) -> anyhow::Result<()> {
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

    pub fn pointer_axis(
        &mut self,
        horizontal: f64,
        vertical: f64,
        time: u64,
    ) -> anyhow::Result<()> {
        let result = unsafe {
            let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
            event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
            event.phase = bindings::FlutterPointerPhase_kHover;
            event.signal_kind = bindings::FlutterPointerSignalKind_kFlutterPointerSignalKindScroll;
            event.scroll_delta_x = horizontal as f64;
            event.scroll_delta_y = vertical as f64;
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
        time: u64,
        button: PointerButtons,
        state: KeyState,
    ) -> anyhow::Result<()> {
        let result = unsafe {
            let mut event = std::mem::zeroed::<bindings::FlutterPointerEvent>();
            event.struct_size = std::mem::size_of::<bindings::FlutterPointerEvent>();
            event.phase = if state == KeyState::Pressed {
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

    pub fn key_press(
        &mut self,
        symbol: xkeysym::Keysym,
        time: u32,
        state: KeyState,
    ) -> anyhow::Result<()> {
        let result = unsafe {
            let mut event = std::mem::zeroed::<bindings::FlutterKeyEvent>();
            event.struct_size = std::mem::size_of::<bindings::FlutterKeyEvent>();

            if state == KeyState::Pressed {
                event.type_ = bindings::FlutterKeyEventType_kFlutterKeyEventTypeDown;
                event.character = symbol.name().unwrap().as_bytes().as_ptr() as *const i8;
            } else {
                event.type_ = bindings::FlutterKeyEventType_kFlutterKeyEventTypeUp;
            }
            event.synthesized = false;
            event.timestamp = time as f64;
            event.logical = translate_logical_key(symbol).unwrap();
            event.physical = translate_physical_key(symbol).unwrap();

            bindings::FlutterEngineSendKeyEvent(
                self.inner,
                &event,
                Some(keydata_callback),
                std::ptr::null_mut(),
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

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}
