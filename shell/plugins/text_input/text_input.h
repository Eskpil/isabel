#ifndef ISABEL_SHELL_PLUGIN_TEXT_INPUT_H
#define ISABEL_SHELL_PLUGIN_TEXT_INPUT_H

#include <memory>

#include "shell/dispatcher.h"
#include "shell/plugins/plugin.h"

namespace shell::plugins {
class TextInput : public Plugin {
public:
  TextInput(std::shared_ptr<Dispatcher>);

  void handle_key_event(uint32_t, bool);

  std::string_view name() { return "text_input"; }

private:
  void handle_message(const FlutterPlatformMessage *);
};
} // namespace shell::plugins

#endif // ISABEL_SHELL_PLUGIN_TEXT_INPUT_H
