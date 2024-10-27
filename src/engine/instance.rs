use std::sync::{Arc, Mutex};
use std::{cell::RefCell, rc::Rc};

use crate::{backend::Backend, shell::app::Application};
use crate::{shell::LoopHandle, tasks::TaskRunner};

use super::{
    builtin::{self},
    engine::Engine,
    Bundle, InstanceError, Plugin, Shell,
};

pub struct Instance {
    engine: super::engine::Engine,
}
impl Instance {
    pub fn new(backend: Arc<Mutex<Backend>>, bundle: Bundle) -> anyhow::Result<Self> {
        let engine = Engine::new(backend, bundle.assets, bundle.icu_data)?;
        let mut instance = Self { engine };

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

        Ok(instance)
    }
    pub fn with(&mut self, plugin: Box<dyn Plugin>) {
        self.engine.extend_with(plugin);
    }

    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    pub fn run<T>(
        &mut self,
        handle: LoopHandle<'static, Application>,
        window: Rc<RefCell<T>>,
    ) -> anyhow::Result<()>
    where
        T: Shell,
    {
        let task_runner = TaskRunner::new(handle.clone());
        self.engine.add_platform_task_runner(task_runner);

        self.engine.run()?;
        self.engine.preload_plugins(window.clone())?;

        Ok(())
    }
}
