use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use isabel_rs::{
    popup::Popup,
    timer::{TimeoutAction, Timer},
    Application, LoopHandle, Shell,
};

use crate::{
    module::{Module, ModuleCapabilities, PopupInfo, ShowRequest},
    ui::Ui,
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Event {
    Run,
}

pub struct Demo {
    popup: Arc<Mutex<Popup>>,
    handle: LoopHandle<'static, Application<'static>>,
}

impl Demo {
    pub fn new(app: isabel_rs::Application<'static>) -> anyhow::Result<Self> {
        let popup = {
            let mut app = app.lock();
            let sm = &mut app.sm;
            sm.prepare_popup()?
        };

        Ok(Self {
            popup,
            handle: app.handle(),
        })
    }
}

impl Module for Demo {
    fn name(&self) -> &str {
        "isabel.io/sidecar/demo"
    }

    fn run(&self) {}

    fn popup_info(&self) -> Option<PopupInfo> {
        Some(PopupInfo {
            anchor: isabel_rs::PopupAnchor::Top,
            gravity: isabel_rs::Gravity::Top,
            width: 200,
            height: 200,
        })
    }

    fn capabilities(&self) -> ModuleCapabilities {
        ModuleCapabilities::POPOP
    }

    fn show(&self, req: ShowRequest) -> anyhow::Result<()> {
        let ShowRequest::Popup(positioner, parent) = req;

        let mut popup = self.popup.lock().unwrap();
        popup.show(positioner, parent)?;

        let surface = popup.surface();

        let handle = self.handle.clone();

        self.handle
            .insert_source(
                Timer::from_duration(Duration::from_nanos(20)),
                move |_, _, _| {
                    let surface = surface.clone();
                    let mut ui = Ui::new(surface.clone()).expect("could not create ui");

                    handle
                        .insert_source(
                            Timer::from_duration(Duration::from_millis(16)),
                            move |_, _, _| {
                                if !surface.clone().present() {
                                    return TimeoutAction::Drop;
                                }

                                ui.run(|ctx| {
                                    egui::CentralPanel::default().show(ctx, |ui| {
                                        ui.heading("Hello, World!");
                                    });
                                });

                                TimeoutAction::ToDuration(Duration::from_millis(16))
                            },
                        )
                        .expect("msg");

                    TimeoutAction::Drop
                },
            )
            .expect("could not insert handle");

        Ok(())
    }

    fn hide(&self) -> anyhow::Result<()> {
        let mut popup = self.popup.lock().unwrap();
        popup.hide()
    }
}
