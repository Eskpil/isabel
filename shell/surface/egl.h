#ifndef ISABEL_SHELL_SURFACE_EGL_H_
#define ISABEL_SHELL_SURFACE_EGL_H_

#include <memory>

#include <EGL/egl.h>
#include <wayland-client.h>

namespace shell::surface {

class EglSurface {
public:
  EglSurface(EGLSurface surface, EGLDisplay display, EGLContext context) : m_display(display), m_surface(surface), m_context(context) {}

  void make_current();
  void clear_current();
  void swap_buffers();
  void *get_proc_address(const char*);

private:
  EGLSurface m_surface;
  EGLDisplay m_display;
  EGLContext m_context;
};

class EglTarget {
public:
  //  TODO: Figure out how to use EGLNativeDisplayType instead of wl_display directly.
  explicit EglTarget(wl_display *);
  ~EglTarget() = default;

  void set_onscreen_window(EGLNativeWindowType window) { m_onscreen_window = window; }
  void set_offscreen_window(EGLNativeWindowType window) { m_offscreen_window = window; }

  void resize_onscreen_window(int, int) const;

  std::unique_ptr<EglSurface> create_onscreen_surface();
  std::unique_ptr<EglSurface> create_offscreen_surface();
private:
  EGLNativeWindowType m_onscreen_window{};
  EGLNativeWindowType m_offscreen_window{};

  EGLConfig m_config;
  EGLDisplay m_display;

  EGLContext m_context, m_resource_context;
};

}

#endif // ISABEL_SHELL_SURFACE_EGL_H_