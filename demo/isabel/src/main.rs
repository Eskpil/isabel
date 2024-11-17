use clap::Parser;
use sidecar::Sidecar;
use std::time::Duration;

use isabel_rs::{
    shell::timer::{TimeoutAction, Timer},
    Anchor, Application, Bundle, EventLoop, Instance, Shell,
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

    let bundle = Bundle {
        assets: args.assets,
        icu_data: args.icudtl,
        aot_elf_path: args.aot,
    };

    let mut app = Application::new(eloop.handle(), bundle.clone())?;

    let layer_surface = {
        let sm = &mut app.lock().sm;
        sm.create_layer(isabel_rs::Layer::Bottom, "isabel-example".to_owned())?
    };
    layer_surface.lock().unwrap().set_size(3440, 48);
    layer_surface.lock().unwrap().set_anchor(Anchor::BOTTOM);
    layer_surface.lock().unwrap().set_exclusive_zone(48);
    layer_surface
        .lock()
        .unwrap()
        .set_keyboard_interactivity(isabel_rs::KeyboardInteractivity::OnDemand);
    layer_surface.lock().unwrap().commit();

    let mut instance = {
        let layer_surface = layer_surface.lock().unwrap();
        Instance::new(
            layer_surface.surface(),
            app.bundle(),
            app.task_runner(),
            None,
        )?
    };

    let sidecar = Sidecar::new(app.clone())?;
    instance.with(Box::new(sidecar));

    let handle = eloop.handle();
    instance.run(&handle, layer_surface)?;

    loop {
        eloop.dispatch(Duration::from_millis(0), &mut app)?;
        if app.exited() {
            break;
        }
    }

    Ok(())
}
