use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::shell::channel::{channel, Event, Sender};

use crate::shell::{timer::TimeoutAction, timer::Timer, LoopHandle};
use crate::Application;

type Task = (usize, Duration, bindings::FlutterTask);

#[derive(Debug)]
enum Msg {
    Engine(usize, bindings::FlutterEngine),
    Task(Task),
}

fn duration_until_target(target_nanos: u64) -> Duration {
    // Convert the target nanoseconds to SystemTime
    let target_time = UNIX_EPOCH + Duration::from_nanos(target_nanos);

    // Get the current time as SystemTime
    let now = SystemTime::now();

    // Calculate the duration until the target time
    match target_time.duration_since(now) {
        Ok(duration) => duration,
        Err(_) => Duration::from_secs(0), // If the target time is in the past, return 0 duration
    }
}

struct TaskRunnerInner {
    pub map: HashMap<usize, bindings::FlutterEngine>,
    counter: usize,
}

#[derive(Clone)]
pub struct TaskRunner {
    inner: Arc<Mutex<TaskRunnerInner>>,
    tx: Sender<Msg>,
    main_thread_id: thread::ThreadId,
}

#[derive(Debug)]
pub struct TaskRunnerClient {
    tx: Sender<Msg>,
    id: usize,
    main_thread_id: thread::ThreadId,
}

impl TaskRunnerInner {
    pub fn new<'a>(handle: &LoopHandle<'a, Application<'static>>) -> (Self, Sender<Msg>) {
        let (tx, rx) = channel();

        let handle2 = handle.clone();
        handle
            .insert_source(rx, move |e: Event<Msg>, _, a| match e {
                Event::Msg(msg) => match msg {
                    Msg::Engine(id, engine) => {
                        a.task_runner().inner().map.insert(id, engine);
                    }
                    Msg::Task(task) => {
                        handle2
                            .clone()
                            .insert_source(Timer::from_duration(task.1), move |_, _, a| {
                                let engine = a.task_runner().inner().map[&task.0];
                                unsafe { bindings::FlutterEngineRunTask(engine, &task.2) };
                                TimeoutAction::Drop
                            })
                            .unwrap();
                    }
                },
                Event::Closed => println!("task runner channel closed"),
            })
            .unwrap();

        (
            Self {
                counter: 0,
                map: HashMap::new(),
            },
            tx,
        )
    }

    fn next_id(&mut self) -> usize {
        let id = self.counter.clone();
        self.counter += 1;
        id
    }
}

impl TaskRunner {
    pub fn new<'a>(handle: &LoopHandle<'a, Application<'static>>) -> anyhow::Result<Self> {
        let (inner, tx) = TaskRunnerInner::new(handle);

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
            tx,
            main_thread_id: thread::current().id(),
        })
    }

    fn inner(&self) -> MutexGuard<'_, TaskRunnerInner> {
        self.inner.lock().unwrap()
    }

    pub fn fork(&self) -> TaskRunnerClient {
        TaskRunnerClient {
            tx: self.tx.clone(),
            id: self.inner.lock().unwrap().next_id(),
            main_thread_id: self.main_thread_id.clone(),
        }
    }
}

impl TaskRunnerClient {
    pub fn set_engine(&mut self, engine: bindings::FlutterEngine) {
        self.tx.send(Msg::Engine(self.id, engine)).unwrap();
    }

    pub unsafe extern "C" fn runs_on_thread(data: *mut std::ffi::c_void) -> bool {
        let runner = unsafe { &*(data as *const Self) as &Self };
        runner.main_thread_id == thread::current().id()
    }

    pub unsafe extern "C" fn post_task(
        task: bindings::FlutterTask,
        target_time: u64,
        data: *mut std::ffi::c_void,
    ) {
        let runner = unsafe { &*(data as *const Self) as &Self };
        match runner.tx.send(Msg::Task((
            runner.id,
            duration_until_target(target_time),
            task,
        ))) {
            Ok(_) => {}
            Err(e) => {
                println!("{:?}", e.to_string());
            }
        }
    }
}
