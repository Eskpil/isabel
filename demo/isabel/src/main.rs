use clap::Parser;
use std::time::Duration;

use isabel_rs::{Config, EventLoop, Instance, Shell, Window};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    assets: String,

    #[arg(short, long)]
    icudtl: String,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();
    let mut eloop = EventLoop::try_new()?;

    let (tx, rx) = isabel_rs::channels();
    let mut window = Window::create(tx, "hello", 1240, 540, &mut eloop)?;

    let config = Config {
        assets: args.assets,
        icu_data: args.icudtl,
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
