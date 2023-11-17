#ifndef ISABEL_SHELL_WINDOW_H_
#define ISABEL_SHELL_WINDOW_H_

#include <string>

#include <aylin/aylin.h>

#include "shell/application.h"

namespace shell {

struct WindowOptions {
  int width, height;
  std::string_view title;

  std::string_view icudtl_path;
  std::string_view flutter_assets_path;
};

class Window : public Target {
public:
  explicit Window(WindowOptions options);
  ~Window() = default;

  void set_title(std::string_view);
private:
  static const struct aylin_shell_listener k_window_listener;

  void configure(Application *) override;

  WindowOptions m_options { 0 };

  bool first_frame { true };

  struct aylin_shell *m_shell;
  Application *m_application;

  int m_width, m_height;
};
}

#endif // ISABEL_SHELL_WINDOW_H_