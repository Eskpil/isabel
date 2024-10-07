use clap::Parser;
use std::time::Duration;

use isabel_rs::{Config, EventLoop, Instance, Shell, Window};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut eloop = EventLoop::try_new()?;

    let (tx, rx) = isabel_rs::channels();
    let mut window = Window::create(tx, "hello", 1240, 540, &mut eloop)?;

    let config = Config {
        assets: String::from("/home/linus/repos/github.com/isabel-rs/demo/build/isabel/x86-64/bundle/data/flutter_assets"),
        icu_data: String::from("./icudtl.dat"),
        aot_elf_path: None,
    };
    let mut instance = Instance::new(window.backend()?, config)?;

    instance.run(eloop.handle(), rx)?;
    window.set_instance(instance);

    loop {
        eloop.dispatch(Duration::from_millis(16), &mut window)?;
        if window.exited() {
            return Ok(());
        }
    }
}
