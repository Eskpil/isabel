// TODO: Support vulkan

extern crate khronos_egl as egl;

use std::sync::{Arc, Mutex};

use egl::API as egl;
use khronos_egl::Display;

use anyhow::Error;
use egl::NativeDisplayType;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

struct SurfaceInner {
    context: egl::Context,
    surface: egl::Surface,
    platform: wayland_egl::WlEglSurface,

    width: usize,
    height: usize,
}

#[derive(Clone)]
pub struct Surface {
    display: egl::Display,
    inner: Arc<Mutex<Option<SurfaceInner>>>,
}

fn create_context(display: egl::Display) -> anyhow::Result<(egl::Context, egl::Config)> {
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
        egl::ALPHA_SIZE,
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

#[derive(Clone, Copy)]
pub struct Backend {
    display: Display,
}

impl Backend {
    pub fn global_prepare(display_handle: &RawDisplayHandle) -> anyhow::Result<Self> {
        let display_ptr = match display_handle {
            RawDisplayHandle::Wayland(w) => w.display.as_ptr() as *mut _ as NativeDisplayType,
            o => {
                return Err(Error::msg(format!(
                    "support for {:?} not implemeted yet",
                    o
                )))
            }
        };

        egl.bind_api(egl::OPENGL_ES_API)?;

        let display = unsafe { egl.get_display(display_ptr).unwrap() };
        egl.initialize(display)?;

        Ok(Self { display })
    }

    pub fn get_proc_address(procname: &str) -> Option<extern "system" fn()> {
        egl.get_proc_address(procname)
    }
}

impl SurfaceInner {
    pub fn new(
        display: Display,
        window_handle: &RawWindowHandle,
        width: usize,
        height: usize,
    ) -> anyhow::Result<Self> {
        let window_ptr = match window_handle {
            RawWindowHandle::Wayland(w) => w.surface.as_ptr(),
            o => {
                return Err(Error::msg(format!(
                    "support for {:?} not implemeted yet",
                    o
                )))
            }
        };

        let egl_surface = unsafe {
            wayland_egl::WlEglSurface::new_from_raw(
                window_ptr as *mut _,
                width as i32,
                height as i32,
            )
        }?;

        egl_surface.resize(width as i32, height as i32, 0, 0);

        let (context, config) = create_context(display)?;

        let surface = unsafe {
            egl.create_window_surface(display, config, egl_surface.ptr() as *mut _, None)
        }?;

        Ok(SurfaceInner {
            context,
            surface,
            platform: egl_surface,

            width,
            height,
        })
    }
}

impl Surface {
    pub fn empty(backend: Backend) -> Self {
        Self {
            display: backend.display,
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub fn new(
        backend: Backend,
        window_handle: &RawWindowHandle,
        width: usize,
        height: usize,
    ) -> anyhow::Result<Self> {
        Ok(Surface {
            display: backend.display,
            inner: Arc::new(Mutex::new(Some(SurfaceInner::new(
                backend.display,
                window_handle,
                width,
                height,
            )?))),
        })
    }

    pub fn present(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.is_some()
    }

    pub fn clear(&mut self) {
        let mut inner = self.inner.lock().unwrap();
        *inner = None;
    }

    pub fn recreate(
        &mut self,
        window_handle: &RawWindowHandle,
        width: usize,
        height: usize,
    ) -> anyhow::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        *inner = Some(SurfaceInner::new(
            self.display,
            window_handle,
            width,
            height,
        )?);
        Ok(())
    }

    pub fn make_current(&self) -> anyhow::Result<()> {
        let display = self.display;
        match &*self.inner.lock().unwrap() {
            Some(inner) => {
                egl.make_current(
                    display,
                    Some(inner.surface),
                    Some(inner.surface),
                    Some(inner.context),
                )?;
            }
            None => {}
        }

        Ok(())
    }

    pub fn dimensions(&self) -> Option<[usize; 2]> {
        let inner = self.inner.lock().unwrap();
        let inner = inner.as_ref().unwrap();
        Some([inner.width, inner.height])
    }

    pub fn resize(&self, width: usize, height: usize, dx: usize, dy: usize) -> anyhow::Result<()> {
        match &mut *self.inner.lock().unwrap() {
            Some(inner) => {
                inner.width = width;
                inner.height = height;
                inner
                    .platform
                    .resize(width as i32, height as i32, dx as i32, dy as i32);
            }
            None => {}
        }

        Ok(())
    }

    pub fn clear_current(&self) -> anyhow::Result<()> {
        egl.make_current(self.display, None, None, None)?;
        Ok(())
    }

    pub fn swap_buffers(&self) -> anyhow::Result<()> {
        let display = self.display;
        match &*self.inner.lock().unwrap() {
            Some(inner) => {
                egl.swap_buffers(display, inner.surface)?;
            }
            None => {}
        }
        Ok(())
    }
}

unsafe impl Send for Backend {}
unsafe impl Send for Surface {}
unsafe impl Send for SurfaceInner {}
