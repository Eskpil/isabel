#ifndef ISABEL_SHELL_PLUGIN_DECORATIONS_H
#define ISABEL_SHELL_PLUGIN_DECORATIONS_H

#include "shell/plugins/plugin.h"
#include "shell/target.h"

namespace shell::plugins {

class Decorations : public Plugin {
public:
  Decorations(std::shared_ptr<Dispatcher>, shell::Target *);

  std::string_view name() { return "decorations"; }

  void primary_pressed(uint32_t);

private:
  void handle_message(const FlutterPlatformMessage *);

  shell::Target *m_target;
};

} // namespace shell::plugins

#endif // ISABEL_SHELL_PLUGIN_DECORATIONS_H
