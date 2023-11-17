#include <vector>

#include <EGL/egl.h>
#include <wayland-egl.h>

#include "shell/surface/egl.h"
#include "shell/log.h"

namespace shell::surface {

std::string get_egl_error_cause() {
  static const std::vector<std::pair<EGLint, std::string>> table = {
      {EGL_SUCCESS, "EGL_SUCCESS"},
      {EGL_NOT_INITIALIZED, "EGL_NOT_INITIALIZED"},
      {EGL_BAD_ACCESS, "EGL_BAD_ACCESS"},
      {EGL_BAD_ALLOC, "EGL_BAD_ALLOC"},
      {EGL_BAD_ATTRIBUTE, "EGL_BAD_ATTRIBUTE"},
      {EGL_BAD_CONTEXT, "EGL_BAD_CONTEXT"},
      {EGL_BAD_CONFIG, "EGL_BAD_CONFIG"},
      {EGL_BAD_CURRENT_SURFACE, "EGL_BAD_CURRENT_SURFACE"},
      {EGL_BAD_DISPLAY, "EGL_BAD_DISPLAY"},
      {EGL_BAD_SURFACE, "EGL_BAD_SURFACE"},
      {EGL_BAD_MATCH, "EGL_BAD_MATCH"},
      {EGL_BAD_PARAMETER, "EGL_BAD_PARAMETER"},
      {EGL_BAD_NATIVE_PIXMAP, "EGL_BAD_NATIVE_PIXMAP"},
      {EGL_BAD_NATIVE_WINDOW, "EGL_BAD_NATIVE_WINDOW"},
      {EGL_CONTEXT_LOST, "EGL_CONTEXT_LOST"},
  };

  auto egl_error = eglGetError();
  for (const auto& t : table) {
    if (egl_error == t.first) {
      return std::string("eglGetError: " + t.second);
    }
  }

  return {};
}

void EglSurface::clear_current() {
  if (eglGetCurrentContext() != m_context) {
    return;
  }

  if (eglMakeCurrent(m_display, EGL_NO_SURFACE, EGL_NO_SURFACE,
                     EGL_NO_CONTEXT) != EGL_TRUE) {
    log::error("failed to clear egl context: ({})", get_egl_error_cause());
    return;
  }
}

void EglSurface::make_current() {
  if (eglMakeCurrent(m_display, m_surface, m_surface, m_context) != EGL_TRUE) {
    log::error("could not make egl context current: ({})", get_egl_error_cause());
  }

  if (eglSwapInterval(m_display, 0) != EGL_TRUE) {
    log::error("failed to eglSwapInterval(Free): ({})", get_egl_error_cause());
  }
}



void *EglSurface::get_proc_address(const char *name) {
  auto address = eglGetProcAddress(name);
  if (!address) {
    log::fatal("could not get proc address");
  }

  return reinterpret_cast<void*>(address);

}

void EglSurface::swap_buffers() {
  if (eglSwapBuffers(m_display, m_surface) != EGL_TRUE) {
    log::error("could not swap buffers: ({})", get_egl_error_cause());
  }
}

std::unique_ptr<EglSurface> EglTarget::create_onscreen_surface() {
  const EGLint attribs[] = {EGL_NONE};
  EGLSurface surface = eglCreateWindowSurface(m_display, m_config,
                                              m_onscreen_window, attribs);
  if (surface == EGL_NO_SURFACE) {
    log::fatal("could not create egl_surface: ({})", get_egl_error_cause());
  }

  return std::make_unique<EglSurface>(surface, m_display, m_context);
}

std::unique_ptr<EglSurface> EglTarget::create_offscreen_surface() {
  const EGLint attribs[] = {EGL_NONE};
  EGLSurface surface = eglCreateWindowSurface(
      m_display, m_config, m_offscreen_window, attribs);
  if (surface == EGL_NO_SURFACE) {
    log::fatal("could not create egl_surface: ({})", get_egl_error_cause());
  }

  return std::make_unique<EglSurface>(surface, m_display, m_resource_context);
}

void EglTarget::resize_onscreen_window(int width, int height) const {
  wl_egl_window_resize(reinterpret_cast<wl_egl_window *>(m_onscreen_window), width, height, 0, 0);
}

EglTarget::EglTarget(wl_display *native_display) {
  m_display = eglGetDisplay(native_display);
  if (m_display == EGL_NO_DISPLAY) {
    log::fatal("could not get egl display: ({})", get_egl_error_cause());
  }

  if (eglInitialize(m_display, nullptr, nullptr) != EGL_TRUE) {
    log::fatal("failed to initialize the egl display: ({})", get_egl_error_cause());
  }

  if (eglBindAPI(EGL_OPENGL_ES_API) != EGL_TRUE) {
    log::fatal("failed to bind the egl api: ({})", get_egl_error_cause());
  }


  EGLint config_count = 0;
  const EGLint attribs[] = {
    // clang-format off
    EGL_SURFACE_TYPE,    EGL_WINDOW_BIT,
    EGL_RENDERABLE_TYPE, EGL_OPENGL_ES2_BIT,
    EGL_RED_SIZE,        8,
    EGL_GREEN_SIZE,      8,
    EGL_BLUE_SIZE,       8,
#if defined(ENABLE_EGL_ALPHA_COMPONENT_OF_COLOR_BUFFER)
    EGL_ALPHA_SIZE,      8,
#endif
    EGL_DEPTH_SIZE,      0,
    EGL_STENCIL_SIZE,    0,
    EGL_NONE
    // clang-format on
  };

  if (eglChooseConfig(m_display, attribs, &m_config, 1,
                      &config_count) != EGL_TRUE) {
    log::fatal("failed to choose egl surface config: ({})", get_egl_error_cause());
  }

  if (config_count == 0 || m_config == nullptr) {
    log::fatal("no matching config: ({})", get_egl_error_cause());
  }

  {
    const EGLint context_attribs[] = {EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE};
    m_context = eglCreateContext(m_display, m_config,
                                EGL_NO_CONTEXT, context_attribs);
    if (m_context == EGL_NO_CONTEXT) {
      log::fatal("failed to create an onscreen context: ({})", get_egl_error_cause());
    }

    m_resource_context =
        eglCreateContext(m_display, m_config, m_context, context_attribs);
    if (m_resource_context == EGL_NO_CONTEXT) {
      log::fatal("failed to create an offscreen resource context: ({})", get_egl_error_cause());
    }
  }
}
}