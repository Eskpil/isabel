use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bindings::FlutterEngineRunTask;

use crate::shell::channel::{channel, Event, Sender};

use crate::shell::{timer::TimeoutAction, timer::Timer, LoopHandle};
use crate::Application;

type Task = (Duration, bindings::FlutterTask);
enum Msg {
    Engine(bindings::FlutterEngine),
    Task(Task),
}

pub struct TaskRunner {
    tx: Sender<Msg>,
    main_thread_id: thread::ThreadId,
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

impl TaskRunner {
    pub fn new<'a>(handle: LoopHandle<'a, Application>) -> Self {
        let (tx, rx) = channel();

        let mut engine = std::ptr::null_mut();
        let handle2 = handle.clone();
        handle
            .insert_source(rx, move |e: Event<Msg>, _, _| {
                if let Event::Msg(msg) = e {
                    match msg {
                        Msg::Engine(e) => engine = e,
                        Msg::Task(t) => {
                            if engine.is_null() {
                                return;
                            }

                            handle2
                                .clone()
                                .insert_source(Timer::from_duration(t.0), move |_, _, _| {
                                    unsafe { FlutterEngineRunTask(engine, &t.1) };
                                    TimeoutAction::Drop
                                })
                                .unwrap();
                        }
                    }
                }
            })
            .unwrap();

        Self {
            tx,

            main_thread_id: std::thread::current().id(),
        }
    }

    pub fn set_engine(&mut self, engine: bindings::FlutterEngine) {
        self.tx.send(Msg::Engine(engine)).unwrap();
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
        runner
            .tx
            .send(Msg::Task((duration_until_target(target_time), task)))
            .unwrap();
    }
}
