
#ifndef ISABEL_SHELL_PLUGIN_H
#define ISABEL_SHELL_PLUGIN_H

#include <memory>

#include "shell/dispatcher.h"

namespace shell::plugins {
class Plugin {
public:
  Plugin(std::shared_ptr<Dispatcher> dispatcher) : m_dispatcher(dispatcher) {}
  virtual ~Plugin() = default;

  virtual std::string_view name() = 0;

  std::shared_ptr<Dispatcher> dispatcher() { return m_dispatcher; }

private:
  std::shared_ptr<Dispatcher> m_dispatcher;
};
} // namespace shell::plugins

#endif // ISABEL_SHELL_PLUGIN_H
