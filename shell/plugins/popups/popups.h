#ifndef ISABEL_SHELL_PLUGINS_POPUPS_H
#define ISABEL_SHELL_PLUGINS_POPUPS_H

#include "shell/plugins/plugin.h"

namespace shell::plugins {
class Popups : public Plugin {
public:
  explicit Popups(std::shared_ptr<Dispatcher>);

  std::string_view name() override { return "popups"; }

private:
  void handle_message(const FlutterPlatformMessage *);
};
} // namespace shell::plugins

#endif // ISABEL_SHELL_PLUGINS_POPUPS_H
