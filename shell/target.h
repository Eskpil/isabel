#ifndef ISABEL_SHELL_TARGET_H
#define ISABEL_SHELL_TARGET_H

#include <memory>
#include <vector>

#include "shell/dispatcher.h"
#include "shell/flutter/embedder.h"
#include "shell/surface/egl.h"
#include "shell/task_runner.h"

#include "shell/plugins/plugin.h"

#include "shell/log.h"

namespace shell {

class Application;

class PluginManager {
  using PluginEntry = std::shared_ptr<plugins::Plugin>;

public:
  PluginManager() : m_plugins(std::vector<PluginEntry>()){};
  ~PluginManager() = default;

  template <typename T, typename... Args> void create(Args &&...args) {
    auto plugin = std::make_shared<T>(std::forward<Args>(args)...);
    m_plugins.push_back(plugin);
  }

  template <typename T> std::shared_ptr<T> get(std::string_view name) {
    for (const auto &plugin : m_plugins) {
      if (plugin->name() == name) {
        auto derivedPlugin = std::dynamic_pointer_cast<T>(plugin);
        if (derivedPlugin) {
          return derivedPlugin;
        } else {
          log::fatal("plugin ({}) is not of type ({})", name, typeid(T).name());
          return nullptr;
        }
      }
    }

    log::fatal("could not find plugin: {}", name);

    return nullptr;
  };

private:
  std::vector<PluginEntry> m_plugins;
};

class Target {
  friend class Application;
  friend class Window;

  Target(std::string_view, std::string_view);

  void run();

private:
  virtual void configure(Application *) = 0;

  void create_platform_listeners();

  void perform_tasks();

  void update_window_metrics(int, int, int);

  void handle_key_pressed(uint32_t keycode, bool pressed, uint32_t serial,
                          size_t timestamp);

  void handle_pointer_motion(double x, double y, size_t timestamp);
  void handle_pointer_enter(double x, double y);
  void handle_pointer_leave();
  void handle_pointer_button(double x, double y, bool pressed,
                             FlutterPointerMouseButtons button,
                             size_t timestamp);

  bool m_configured{false};
  bool m_engine_is_running{false};
  FlutterEngine m_engine;

  std::unique_ptr<surface::EglTarget> m_egl_target;

  std::unique_ptr<surface::EglSurface> m_primary_surface;
  std::unique_ptr<surface::EglSurface> m_resource_surface;

  // TODO: Separate out into some kind of cache/struct.
  FlutterPointerPhase m_last_pointer_phase;
  FlutterPointerMouseButtons m_last_pointer_button;

  std::unique_ptr<TaskRunner> m_task_runner;
  std::unique_ptr<PluginManager> m_plugins;

  std::shared_ptr<Dispatcher> m_dispatcher;
};
} // namespace shell

#endif // ISABEL_SHELL_TARGET_H
