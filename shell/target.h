#ifndef ISABEL_SHELL_TARGET_H
#define ISABEL_SHELL_TARGET_H

#include <memory>

#include "shell/surface/egl.h"
#include "shell/flutter/embedder.h"

namespace shell {

class Application;

struct Buffer {
  void *pixels;
  size_t stride;
  size_t height;
};

class Target {
  friend class Application;
  friend class Window;

  Target();

  void run();
private:
  virtual void configure(Application *) = 0;

  void update_window_metrics(int, int, int);

  bool m_configured { false };
  bool m_engine_is_running { false };
  FlutterEngine m_engine;

  std::unique_ptr<surface::EglTarget> m_egl_target;
  std::unique_ptr<surface::EglSurface> m_egl_surface;

  Buffer m_buffer;
  bool m_buffer_full { false };
};
}

#endif // ISABEL_SHELL_TARGET_H
