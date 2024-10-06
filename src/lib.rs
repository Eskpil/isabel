pub mod backend;
pub mod codec;
pub mod engine;
pub mod shell;
pub mod tasks;
pub mod textmodel;

pub use engine::{Config, Instance, Shell};
pub use shell::{EventLoop, Window};

pub fn channels() -> (
    shell::channel::Sender<engine::Event>,
    shell::channel::Channel<engine::Event>,
) {
    shell::channel::channel::<engine::Event>()
}
