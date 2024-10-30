use std::{cell::RefCell, rc::Rc};

use smithay_client_toolkit::reexports::calloop::channel::Sender;

use crate::{
    engine::{EngineRequest, Plugin},
    Shell,
};

#[derive(Clone)]
pub struct Lifecycle {
    shell: Option<Rc<RefCell<dyn Shell>>>,
    tx: Option<Sender<EngineRequest>>,
}

impl Lifecycle {
    pub fn new() -> Self {
        Self {
            shell: None,
            tx: None,
        }
    }

    pub fn app_is_inactive(&mut self) {
        self.send("AppLifecycleState.inactive");
    }
    pub fn app_is_resumed(&mut self) {
        self.send("AppLifecycleState.resumed");
    }
    pub fn app_is_paused(&mut self) {
        self.send("AppLifecycleState.paused");
    }
    pub fn app_is_detached(&mut self) {
        self.send("AppLifecycleState.detached");
    }

    fn send(&mut self, message: &str) {
        self.tx
            .as_ref()
            .unwrap()
            .send(EngineRequest::Publish {
                channel: self.on().to_owned(),
                data: message.as_bytes().to_vec(),
            })
            .unwrap();
    }
}

impl crate::engine::Plugin for Lifecycle {
    fn init(
        &mut self,
        shell: Rc<RefCell<dyn Shell>>,
        tx: Sender<EngineRequest>,
    ) -> anyhow::Result<()> {
        self.shell = Some(shell);
        self.tx = Some(tx);
        Ok(())
    }

    fn on(&self) -> &str {
        "flutter/lifecycle"
    }
}
