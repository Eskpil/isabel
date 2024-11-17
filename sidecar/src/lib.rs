mod builtin;
mod ipc;
mod module;
mod ui;

use ipc::{SidecarRequest, SidecarRequestPopup};
use isabel_rs::engine::InstanceState;
use isabel_rs::{channel::Sender, EngineRequest};
use module::{Module, ModuleCapabilities, ShowRequest};
use std::sync::{Arc, Mutex};

pub struct Sidecar {
    tx: Option<Sender<EngineRequest>>,
    shell: Option<Arc<Mutex<dyn isabel_rs::Shell>>>,

    modules: Vec<Box<dyn Module>>,
}

impl Sidecar {
    pub fn new(app: isabel_rs::Application<'static>) -> anyhow::Result<Self> {
        let mut sidecar = Self {
            tx: None,
            shell: None,
            modules: vec![],
        };

        sidecar
            .modules
            .push(Box::new(builtin::demo::Demo::new(app.clone())?));

        Ok(sidecar)
    }

    fn handle_request_popup(
        &mut self,
        app: &mut isabel_rs::Application<'static>,
        req: SidecarRequestPopup,
    ) -> anyhow::Result<()> {
        // TODO: Handle errors more gracefully here. I.E map them to a custom SidecarError type
        let module = self
            .modules
            .iter_mut()
            .find(|m| m.name() == req.name.as_str())
            .unwrap();

        if !module.capabilities().contains(ModuleCapabilities::POPOP) {
            todo!("implemenet error here");
        }

        let sm = &mut app.lock().sm;

        let positioner = sm.get_positioner()?;
        positioner.set_anchor_rect(
            req.anchor_x,
            req.anchor_y,
            req.anchor_width,
            req.anchor_height,
        );

        let parent_info = self.shell.as_ref().unwrap().lock().unwrap().parent_info();
        positioner.set_parent_size(parent_info.width, parent_info.height);

        if parent_info.last_configure > 0 {
            positioner.set_parent_configure(parent_info.last_configure);
        }

        let popup_info = module.popup_info().unwrap();
        positioner.set_anchor(popup_info.anchor);
        positioner.set_gravity(popup_info.gravity);
        positioner.set_size(popup_info.width, popup_info.height);
        positioner.set_reactive();
        positioner.set_offest(0, -12);

        match module.show(ShowRequest::Popup(positioner, parent_info.parent)) {
            Ok(_) => Ok(()),
            Err(e) => {
                if !e.to_string().contains("popup is already visible") {
                    Err(e)
                } else {
                    module.hide()
                }
            }
        }?;

        Ok(())
    }
}

impl isabel_rs::Plugin for Sidecar {
    fn on(&self) -> &str {
        "isabel/sidecar"
    }

    fn state_changed(&mut self, state: InstanceState) -> anyhow::Result<()> {
        match state {
            InstanceState::Running => {
                for module in &mut self.modules {
                    module.run();
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn init(
        &mut self,
        shell: Arc<Mutex<dyn isabel_rs::Shell>>,
        tx: Sender<EngineRequest>,
    ) -> anyhow::Result<()> {
        self.shell = Some(shell);
        self.tx = Some(tx);
        Ok(())
    }

    fn handle(
        &mut self,
        app: &mut isabel_rs::Application<'static>,
        data: Vec<u8>,
    ) -> anyhow::Result<()> {
        let req: SidecarRequest = serde_json::de::from_slice(&data)?;

        match req {
            SidecarRequest::Popup(req) => self.handle_request_popup(app, req)?,
        }

        Ok(())
    }
}

unsafe impl Send for Sidecar {}
unsafe impl Sync for Sidecar {}
