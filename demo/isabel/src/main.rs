use clap::Parser;
use std::time::Duration;

use isabel_rs::{Application, Bundle, EventLoop, Instance, Shell, Window};

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

    let bundle = Bundle {
        assets: args.assets,
        icu_data: args.icudtl,
        aot_elf_path: args.aot,
    };

    let mut app = Application::new(eloop.handle(), bundle.clone())?;

    let (display_tx, display_rx) = Instance::channels();

    let mut window = Window::new(&mut app, display_tx, 720, 480)?;

    let instance = Instance::new(
        window.surface(),
        bundle,
        app.task_runner(),
        display_rx,
        None,
    )?;

    window.set_title("Isabel Example");
    window.set_app_id("io.isabel.Example");

    instance.run(&app.handle(), Box::new(window))?;

    loop {
        eloop.dispatch(Duration::from_millis(20), &mut app)?;
        if app.exited() {
            break;
        }
    }

    Ok(())
}
