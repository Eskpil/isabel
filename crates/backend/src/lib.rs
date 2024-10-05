extern crate khronos_egl as egl;

use anyhow::Error;
use egl::{NativeDisplayType, NativeWindowType};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub type SurfaceId = usize;

#[derive(Debug)]
#[repr(C)]
pub struct Backend {
    instance: egl::DynamicInstance,
    display: egl::Display,
    config: egl::Config,
    context: egl::Context,
    surfaces: Vec<egl::Surface>,
}

fn create_context(
    egl: &egl::DynamicInstance,
    display: egl::Display,
) -> anyhow::Result<(egl::Context, egl::Config)> {
    let attributes = [
        egl::SURFACE_TYPE,
        egl::WINDOW_BIT,
        egl::RENDERABLE_TYPE,
        egl::OPENGL_ES2_BIT,
        egl::RED_SIZE,
        8,
        egl::GREEN_SIZE,
        8,
        egl::BLUE_SIZE,
        8,
        egl::NONE,
    ];

    let config = egl.choose_first_config(display, &attributes)?;
    if config.is_none() {
        return Err(Error::msg("could not find egl config"));
    }

    let config = config.unwrap();

    let context_attributes = [egl::CONTEXT_CLIENT_VERSION, 2, egl::NONE];

    let context = egl.create_context(display, config, None, &context_attributes)?;

    Ok((context, config))
}

impl Backend {
    pub fn new(display_handle: &RawDisplayHandle) -> anyhow::Result<Backend> {
        let instance = unsafe { egl::DynamicInstance::<egl::EGL1_5>::load_required()? };

        instance.bind_api(egl::OPENGL_ES_API)?;

        let display_ptr = match display_handle {
            RawDisplayHandle::Wayland(w) => w.display.as_ptr() as *mut _ as NativeDisplayType,
            o => {
                return Err(Error::msg(format!(
                    "support for {:?} not implemeted yet",
                    o
                )))
            }
        };

        let display = unsafe { instance.get_display(display_ptr).unwrap() };
        instance.initialize(display)?;

        let (context, config) = create_context(&instance, display)?;

        Ok(Backend {
            instance,
            display,
            config,
            context,
            surfaces: Vec::new(),
        })
    }

    pub fn get_proc_address(&self, procname: &str) -> Option<extern "system" fn()> {
        self.instance.get_proc_address(procname)
    }

    pub fn surface(&mut self, window_handle: &RawWindowHandle) -> anyhow::Result<usize> {
        let window_ptr = match window_handle {
            RawWindowHandle::Wayland(w) => w.surface.as_ptr() as *mut _ as NativeWindowType,
            o => {
                return Err(Error::msg(format!(
                    "support for {:?} not implemeted yet",
                    o
                )))
            }
        };

        let surface = unsafe {
            self.instance
                .create_window_surface(self.display, self.config, window_ptr, None)?
        };

        let id = self.surfaces.len();
        self.surfaces.insert(id, surface);

        Ok(id)
    }

    pub fn clear_current(&self) -> anyhow::Result<()> {
        self.make_current(usize::MAX)
    }

    pub fn make_current(&self, id: usize) -> anyhow::Result<()> {
        let (surface, context) = if id == usize::MAX {
            (None, None)
        } else {
            (Some(self.surfaces[id]), Some(self.context))
        };

        self.instance
            .make_current(self.display, surface, surface, context)?;

        Ok(())
    }

    pub fn swap_buffers(&self, id: usize) -> anyhow::Result<()> {
        if self.instance.get_current_context() != Some(self.context) {
            return Ok(());
        }

        let surface = self.surfaces[id];
        self.instance.swap_buffers(self.display, surface)?;
        Ok(())
    }
}
