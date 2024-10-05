use std::time::Duration;

use engine::{Event, Instance, Shell};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut eloop = shell::EventLoop::try_new()?;

    let (tx, rx) = calloop::channel::channel::<Event>();
    let mut window = shell::Window::create(tx, "hello", 1240, 540, &mut eloop)?;

    let mut instance = Instance::new(window.backend()?)?;

    instance.run(eloop.handle(), rx)?;
    window.set_instance(instance);

    loop {
        eloop.dispatch(Duration::from_millis(16), &mut window)?;
        if window.exited() {
            return Ok(());
        }
    }
}
