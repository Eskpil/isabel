use backend;
use codec;
use engine;
use shell;
use tasks;
use textmodel;

struct BatteryWidget {}
impl BatteryWidget {
    pub fn new() -> Self {}
}

impl widgets::Frame for BatteryWidget {}

pub fn widget_run<T, B>(popup: T, widget: B) -> anyhow::Result<()>
where
    T: Widgetable,
    B: widgets::Frame,
{
}

pub fn main() {
    widget_run(popup, BatteryWidget::new());
}
