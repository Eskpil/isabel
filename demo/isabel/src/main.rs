use clap::Parser;
use std::time::Duration;

use isabel_rs::{Bundle, EventLoop, Instance, Shell};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    assets: String,

    #[arg(short, long)]
    icudtl: String,

    #[arg(long, default_value=None)]
    aot: Option<String>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();
    let mut eloop = EventLoop::try_new()?;

    let mut app = isabel_rs::shell::app::Application::new(&mut eloop)?;
    let window = app.create_window("Demo", 1024, 768)?;

    let config = Bundle {
        assets: args.assets,
        icu_data: args.icudtl,
        aot_elf_path: args.aot,
    };

    let mut instance = {
        let mut window = window.borrow_mut();
        Instance::new(window.backend()?, config)?
    };

    instance.run(eloop.handle(), window.clone())?;
    window.borrow_mut().set_instance(instance);

    loop {
        eloop.dispatch(Duration::from_millis(16), &mut app)?;
        if app.exited() {
            break;
        }
    }

    Ok(())
}
