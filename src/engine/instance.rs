use std::sync::{Arc, Mutex};
use std::{cell::RefCell, rc::Rc};

use smithay_client_toolkit::reexports::calloop::channel::{channel, Event, Sender};

use crate::{backend::Backend, shell::app::Application};
use crate::{shell::LoopHandle, tasks::TaskRunner};

use super::builtin::{Lifecycle, Textinput};
use super::engine::{EngineEvent, EngineSource};
use super::EngineRequest;
use super::{
    builtin::{self},
    engine::Engine,
    Bundle, InstanceError, Plugin, Shell,
};

pub struct Instance {
    engine: Engine,
    source: Option<EngineSource>,

    plugins: Vec<Box<dyn Plugin>>,
}
impl Instance {
    pub fn new(backend: Arc<Mutex<Backend>>, bundle: Bundle) -> anyhow::Result<Self> {
        let engine = Engine::new(backend, bundle.assets, bundle.icu_data)?;
        let mut instance = Self {
            engine: engine.0,
            source: Some(engine.1),

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

    pub fn with(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    pub fn key_press(&mut self, symbol: xkeysym::Keysym) -> anyhow::Result<()> {
        let plugin = self
            .plugins
            .iter_mut()
            .find(|p| p.on() == "flutter/textinput")
            .unwrap();
        let textinput = plugin.downcast_mut::<Textinput>().unwrap();
        textinput.handle_key_event(symbol, true);

        Ok(())
    }

    pub fn preload_plugins(
        &mut self,
        shell: Rc<RefCell<dyn super::Shell>>,
        tx: Sender<EngineRequest>,
    ) -> anyhow::Result<()> {
        for plugin in &mut self.plugins {
            plugin.init(shell.clone(), tx.clone())?;
        }

        Ok(())
    }

    pub fn run<'a, T>(
        mut self,
        handle: &LoopHandle<'a, Application<'static>>,
        window: Rc<RefCell<T>>,
    ) -> anyhow::Result<()>
    where
        T: Shell,
    {
        let task_runner = TaskRunner::new(&handle);
        self.engine.add_platform_task_runner(task_runner);

        let (tx, rx) = channel();
        self.preload_plugins(window.clone(), tx)?;
        self.engine.run()?;

        let engine_source = self.source.take().unwrap();
        let instance = Rc::new(RefCell::new(self));
        let instance2 = instance.clone();

        handle
            .insert_source(engine_source, move |event, _, a| match event {
                EngineEvent::PlatformMessage { channel, data } => {
                    let mut instance = instance2.borrow_mut();
                    let plugin = instance
                        .plugins
                        .iter_mut()
                        .find(|p| p.on().to_owned() == channel);

                    match plugin {
                        Some(plugin) => {
                            plugin.handle(a, data).expect("could not handle message");
                        }
                        None => {}
                    }
                }
            })
            .expect("could not insert");

        let instance3 = instance.clone();
        handle
            .insert_source(rx, move |e, _, _| {
                if let Event::Msg(e) = e {
                    match e {
                        EngineRequest::Publish { channel, data } => {
                            let mut instance = instance3.borrow_mut();
                            instance.engine.publish(channel, data);
                        }
                    }
                }
            })
            .expect("could not insert");

        window.borrow_mut().set_instance(instance);

        Ok(())
    }
}
