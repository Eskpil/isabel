use std::sync::{Arc, Mutex};

use smithay_client_toolkit::reexports::calloop::channel::{
    channel, Channel, Event as ChannelEvent, Sender,
};

use crate::backend::Surface;
use crate::event::Event;
use crate::sm::KeyState;
use crate::Application;
use crate::{shell::LoopHandle, tasks::TaskRunner};

use super::builtin::{Keys, Lifecycle, Textinput};
use super::engine::{EngineEvent, EngineSource};
use super::{
    builtin::{self},
    engine::Engine,
    Bundle, InstanceError, Plugin,
};
use super::{EngineRequest, InstanceState, Shell};

pub struct Instance {
    engine: Engine,
    source: Option<EngineSource>,
    rx: Option<Channel<Event>>,

    plugins: Vec<Box<dyn Plugin>>,
}

impl Instance {
    pub fn channels() -> (Sender<Event>, Channel<Event>) {
        channel()
    }

    pub fn new(
        surface: Surface,
        bundle: Bundle,
        task_runner: TaskRunner,
        rx: Channel<Event>,
        entry: Option<String>,
    ) -> anyhow::Result<Self> {
        let engine = Engine::new(surface, bundle.assets, bundle.icu_data, task_runner, entry)?;
        let mut instance = Self {
            engine: engine.0,
            source: Some(engine.1),
            rx: Some(rx),

            plugins: vec![],
        };

        if instance.engine.runs_aot() {
            if bundle.aot_elf_path.is_none() {
                return Err(InstanceError::MissingAotPath.into());
            }

            instance
                .engine
                .create_aot_data(bundle.aot_elf_path.unwrap())?;
        }

        instance.with(Box::new(builtin::Keys::new()));
        instance.with(Box::new(builtin::Textinput::new()));
        instance.with(Box::new(builtin::Mousecursor::new()));
        instance.with(Box::new(builtin::Lifecycle::new()));

        Ok(instance)
    }

    pub fn hide(&mut self) {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.on() == "flutter/lifecycle")
            .unwrap();
        let lifecycle = plugin.downcast_mut::<Lifecycle>().unwrap();
        lifecycle.app_is_detached();
    }

    pub fn show(&mut self) {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.on() == "flutter/lifecycle")
            .unwrap();
        let lifecycle = plugin.downcast_mut::<Lifecycle>().unwrap();
        lifecycle.app_is_resumed();
    }

    pub fn with(&mut self, plugin: Box<dyn Plugin + Send + Sync>) {
        self.plugins.push(plugin);
    }

    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    pub fn textinput_key_press(&mut self, symbol: xkeysym::Keysym) -> anyhow::Result<()> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.on() == "flutter/textinput")
            .unwrap();
        let textinput = plugin.downcast_mut::<Textinput>().unwrap();
        textinput.handle_key_event(symbol, true);

        Ok(())
    }

    pub fn keyevent_key(&mut self, symbol: xkeysym::Keysym, state: KeyState) -> anyhow::Result<()> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.on() == "isabel/keys")
            .unwrap();
        let keys = plugin.downcast_mut::<Keys>().unwrap();
        keys.send_key(symbol, state);

        Ok(())
    }

    pub fn key_press(
        &mut self,
        symbol: xkeysym::Keysym,
        time: u64,
        state: KeyState,
    ) -> anyhow::Result<()> {
        self.keyevent_key(symbol, state)?;

        if state == KeyState::Released {
            self.textinput_key_press(symbol)?;
        }

        Ok(())
    }

    pub fn preload_plugins(
        &mut self,
        shell: &mut Box<dyn Shell>,
        tx: Sender<EngineRequest>,
    ) -> anyhow::Result<()> {
        for plugin in &mut self.plugins {
            plugin.init(shell, tx.clone())?;
        }

        Ok(())
    }

    fn state_updated(&mut self, state: InstanceState) -> anyhow::Result<()> {
        for plugin in &mut self.plugins {
            plugin.state_changed(state)?;
        }

        Ok(())
    }
    pub fn run<'a>(
        mut self,
        handle: &LoopHandle<'a, Application<'static>>,
        mut shell: Box<dyn Shell>,
    ) -> anyhow::Result<()> {
        let (tx, rx) = channel();
        self.preload_plugins(&mut shell, tx)?;
        self.engine.run()?;
        self.state_updated(InstanceState::Running)?;

        let engine_source = self.source.take().unwrap();

        let instance = Arc::new(Mutex::new(self));

        Self::setup_engine_event_handler(handle, engine_source, instance.clone())?;
        Self::setup_engine_request_handler(handle, rx, instance.clone())?;
        Self::setup_shell_event_handler(handle, instance)?;

        Ok(())
    }

    fn handle_shell_event(
        instance: &mut Self,
        event: Event,
        app: &mut Application<'static>,
    ) -> anyhow::Result<()> {
        match event {
            Event::Exit => {}
            Event::Hide => {}
            Event::Show => {}
            Event::Key {
                state,
                symbol,
                time,
            } => instance.key_press(symbol, time, state)?,
            Event::PointerAxis {
                vertical,
                horizontal,
                time,
            } => instance
                .engine_mut()
                .pointer_axis(horizontal, vertical, time)?,
            Event::PointerButton {
                x,
                y,
                time,
                button,
                state,
            } => instance
                .engine_mut()
                .pointer_button(x, y, time, button, state)?,
            Event::PointerEnter { x, y } => instance.engine_mut().pointer_enter(x, y)?,
            Event::PointerLeave {} => instance.engine_mut().pointer_leave()?,
            Event::PointerMotion { x, y, time } => {
                instance.engine_mut().pointer_motion(x, y, time)?
            }
            Event::Resize {
                width,
                height,
                scale,
            } => instance.engine_mut().resize(width, height)?,
        }

        Ok(())
    }

    fn setup_shell_event_handler<'a>(
        handle: &LoopHandle<'a, Application<'static>>,
        instance: Arc<Mutex<Self>>,
    ) -> anyhow::Result<()> {
        let rx = instance.lock().unwrap().rx.take().unwrap();
        handle
            .insert_source(rx, move |event, _, app| {
                if let ChannelEvent::Msg(e) = event {
                    Self::handle_shell_event(&mut instance.lock().unwrap(), e, app)
                        .expect("failed");
                }
            })
            .expect("insert handle failed");

        Ok(())
    }

    fn setup_engine_event_handler<'a>(
        handle: &LoopHandle<'a, Application<'static>>,
        engine_source: EngineSource,
        instance: Arc<Mutex<Self>>,
    ) -> anyhow::Result<()> {
        handle
            .insert_source(engine_source, move |event, _, app| {
                let EngineEvent::PlatformMessage { channel, data } = event;
                Self::handle_platform_message(&mut instance.lock().unwrap(), channel, data, app);
            })
            .expect("insert failed");

        Ok(())
    }

    fn handle_platform_message(
        instance: &mut Self,
        channel: String,
        data: Vec<u8>,
        app: &mut Application<'static>,
    ) {
        if let Some(plugin) = instance
            .plugins
            .iter_mut()
            .find(|p| p.on().to_owned() == channel)
        {
            plugin.handle(app, data).expect("could not handle message");
        }
    }

    fn setup_engine_request_handler<'a>(
        handle: &LoopHandle<'a, Application<'static>>,
        rx: Channel<EngineRequest>,
        instance: Arc<Mutex<Self>>,
    ) -> anyhow::Result<()> {
        handle
            .insert_source(rx, move |event, _, _| {
                if let ChannelEvent::Msg(EngineRequest::Publish { channel, data }) = event {
                    instance.lock().unwrap().engine.publish(channel, data);
                }
            })
            .expect("insert source failed");

        Ok(())
    }
}
