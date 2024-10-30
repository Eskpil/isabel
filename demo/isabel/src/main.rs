use clap::Parser;
use std::time::Duration;

use isabel_rs::{
    shell::timer::{TimeoutAction, Timer},
    Application, Bundle, EventLoop, Instance, Shell,
};

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

    let mut app = Application::new(eloop.handle())?;
    let window = app.create_window(1024, 768)?;

    window.borrow_mut().set_title("Demo".to_owned());
    window.borrow_mut().set_app_id("io.isabel.demo".to_owned());

    let config = Bundle {
        assets: args.assets,
        icu_data: args.icudtl,
        aot_elf_path: args.aot,
    };

    let instance = {
        let mut window = window.borrow_mut();
        Instance::new(window.backend()?, config)?
    };

    let handle = eloop.handle();
    instance.run(&handle, window.clone())?;

    let window1 = window.clone();
    eloop
        .handle()
        .insert_source(
            Timer::from_duration(Duration::from_secs(5)),
            move |_, _, _| {
                window1.borrow_mut().hide().expect("could not hide window");
                TimeoutAction::Drop
            },
        )
        .unwrap();

    eloop
        .handle()
        .insert_source(
            Timer::from_duration(Duration::from_secs(7)),
            move |_, _, _| {
                window.borrow_mut().show().expect("could not show window");
                TimeoutAction::Drop
            },
        )
        .unwrap();

    loop {
        eloop.dispatch(Duration::from_millis(0), &mut app)?;
        if app.exited() {
            break;
        }
    }

    Ok(())
}
