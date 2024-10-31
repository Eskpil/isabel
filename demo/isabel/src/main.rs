use clap::Parser;
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

    let config = Bundle {
        assets: args.assets,
        icu_data: args.icudtl,
        aot_elf_path: args.aot,
    };

    let mut app = Application::new(eloop.handle())?;
    //let window = app.create_window(1024, 768)?;

    //window.borrow_mut().set_title("Demo".to_owned());
    //window.borrow_mut().set_app_id("io.isabel.demo".to_owned());

    //let instance = {
    //    let mut window = window.borrow_mut();
    //    Instance::new(window.backend()?, config)?
    //};

    //let handle = eloop.handle();
    //instance.run(&handle, window.clone())?;

    let layer_surface = app.create_layer(isabel_rs::Layer::Bottom, "isabel-example".to_owned())?;
    layer_surface.borrow_mut().set_size(3440, 48);
    layer_surface.borrow_mut().set_anchor(Anchor::BOTTOM);
    layer_surface.borrow_mut().set_exclusive_zone(48);
    layer_surface.borrow_mut().commit();

    let instance = {
        let mut layer_surface = layer_surface.borrow_mut();
        Instance::new(layer_surface.backend()?, config)?
    };

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
