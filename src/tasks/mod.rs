use std::cmp::Ordering;
use std::collections;
use std::collections::BinaryHeap;
use std::sync;
use std::sync::atomic::AtomicU64;
use std::thread;

#[derive(Debug)]
struct SendFlutterTask(bindings::FlutterTask);
unsafe impl Send for SendFlutterTask {}

#[derive(Debug)]
pub struct Task {
    order: u64,
    target_time: u64,
    task: SendFlutterTask,
}

#[derive(Debug)]
pub struct TaskRunner {
    main_thread_id: thread::ThreadId,

    task_order: AtomicU64,
    tasks: sync::Mutex<collections::BinaryHeap<Task>>,
}

impl TaskRunner {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            main_thread_id: thread::current().id(),
            tasks: sync::Mutex::new(BinaryHeap::new()),
            task_order: AtomicU64::new(0),
        })
    }

    pub fn process(&mut self, engine: bindings::FlutterEngine) -> bool {
        let mut expired_tasks = vec![];

        {
            let mut tasks = self.tasks.lock().expect("could not lock");
            while !tasks.is_empty() {
                let task = tasks.pop().unwrap();
                expired_tasks.push(task);
            }
        }

        for task in expired_tasks {
            unsafe {
                bindings::FlutterEngineRunTask(engine, &task.task.0);
            }
        }

        true
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
        let task = SendFlutterTask(task);
        let runner = unsafe { &*(data as *const Self) as &Self };

        let order = runner
            .task_order
            .fetch_add(1, sync::atomic::Ordering::Relaxed);

        runner.tasks.lock().unwrap().push(Task {
            target_time,
            task,
            order,
        });
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.target_time == other.target_time {
            if self.order > other.order {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        } else {
            if self.target_time > other.target_time {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Task {}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.target_time == other.target_time && self.order == other.order
    }
}
