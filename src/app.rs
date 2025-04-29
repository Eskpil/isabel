use std::sync::{Arc, Mutex, MutexGuard};

use smithay_client_toolkit::reexports::calloop::LoopHandle;

use crate::{shell, tasks::TaskRunner, Bundle, SurfaceManager};

pub struct ApplicationInner<'a> {
    pub sm: SurfaceManager<'a>,
    bundle: Bundle,

    handle: LoopHandle<'a, Application<'a>>,
    task_runner: TaskRunner,
}

#[derive(Clone)]
pub struct Application<'a> {
    inner: Arc<Mutex<ApplicationInner<'a>>>,
}

impl<'a> ApplicationInner<'a> {
    fn new(handle: LoopHandle<'a, Application<'a>>, bundle: Bundle) -> anyhow::Result<Self>
    where
        'a: 'static,
    {
        let sm = SurfaceManager::new(handle.clone())?;
        let task_runner = TaskRunner::new(&handle.clone())?;

        Ok(Self {
            sm,
            bundle,
            handle,
            task_runner,
        })
    }
}

impl<'a> Application<'a>
where
    'a: 'static,
{
    pub fn new(handle: LoopHandle<'a, Self>, bundle: Bundle) -> anyhow::Result<Self> {
        Ok(Self {
            inner: Arc::new(Mutex::new(ApplicationInner::new(handle, bundle)?)),
        })
    }

    pub fn task_runner(&self) -> TaskRunner {
        self.inner.lock().unwrap().task_runner.clone()
    }

    pub fn lock(&self) -> MutexGuard<'_, ApplicationInner<'a>> {
        self.inner.lock().unwrap()
    }

    pub fn bundle(&self) -> Bundle {
        self.inner.lock().unwrap().bundle.clone()
    }

    pub fn handle(&self) -> LoopHandle<'a, Self> {
        self.inner.lock().unwrap().handle.clone()
    }

    pub fn exited(&self) -> bool {
        self.inner.lock().unwrap().sm.exited()
    }
}

unsafe impl<'a> Send for Application<'a> {}
unsafe impl<'a> Sync for Application<'a> {}
