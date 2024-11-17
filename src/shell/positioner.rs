use smithay_client_toolkit::{
    reexports::protocols::xdg::shell::client::xdg_positioner::{
        Anchor, ConstraintAdjustment, Gravity,
    },
    shell::xdg::XdgPositioner,
};

pub struct Positioner {
    inner: XdgPositioner,
}

impl Positioner {
    pub fn new(inner: XdgPositioner) -> Self {
        Self { inner }
    }

    pub fn set_size(&self, width: usize, height: usize) {
        self.inner.set_size(width as i32, height as i32);
    }

    pub fn set_anchor_rect(&self, x: usize, y: usize, width: usize, height: usize) {
        self.inner
            .set_anchor_rect(x as i32, y as i32, width as i32, height as i32);
    }

    pub fn set_anchor(&self, anchor: Anchor) {
        self.inner.set_anchor(anchor);
    }

    pub fn set_gravity(&self, gravity: Gravity) {
        self.inner.set_gravity(gravity);
    }

    pub fn set_constraint_adjustment(&self, value: ConstraintAdjustment) {
        self.inner.set_constraint_adjustment(value);
    }

    pub fn set_offest(&self, x: i32, y: i32) {
        self.inner.set_offset(x as i32, y as i32);
    }

    pub fn set_reactive(&self) {
        self.inner.set_reactive();
    }

    pub fn set_parent_size(&self, width: usize, height: usize) {
        self.inner.set_parent_size(width as i32, height as i32);
    }

    pub fn set_parent_configure(&self, serial: usize) {
        self.inner.set_parent_configure(serial as u32);
    }

    pub fn build(self) -> XdgPositioner {
        self.inner
    }
}
