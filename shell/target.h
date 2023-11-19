#ifndef ISABEL_SHELL_TARGET_H
#define ISABEL_SHELL_TARGET_H

#include <memory>

#include "shell/surface/egl.h"
#include "shell/flutter/embedder.h"
#include "shell/task_runner.h"

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

  Target(std::string_view, std::string_view);

  void run();
private:
  virtual void configure(Application *) = 0;

  void perform_tasks();

  void update_window_metrics(int, int, int);

  void handle_key_pressed(uint32_t keycode, bool pressed, uint32_t serial, size_t timestamp);

  void handle_pointer_motion(double x, double y, size_t timestamp);
  void handle_pointer_enter(double x, double y);
  void handle_pointer_leave();
  void handle_pointer_button(double x, double y, bool pressed, FlutterPointerMouseButtons button, size_t timestamp);

  bool m_configured { false };
  bool m_engine_is_running { false };
  FlutterEngine m_engine;

  std::unique_ptr<surface::EglTarget> m_egl_target;

  std::unique_ptr<surface::EglSurface> m_primary_surface;
  std::unique_ptr<surface::EglSurface> m_resource_surface;

  // TODO: Separate out into some kind of cache/struct.
  FlutterPointerPhase m_last_pointer_phase;
  FlutterPointerMouseButtons m_last_pointer_button;

  std::unique_ptr<TaskRunner> m_task_runner;

  Buffer m_buffer;
  bool m_buffer_full { false };
};
}

#endif // ISABEL_SHELL_TARGET_H
