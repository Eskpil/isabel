#include "shell/plugins/text_input/text_input.h" //

#include "shell/log.h"

namespace shell::plugins {

static const auto k_channel = "flutter/textinput";

TextInput::TextInput(std::shared_ptr<Dispatcher> dispatch) : Plugin(dispatch) {
  dispatcher()->on(k_channel, [this](const FlutterPlatformMessage *message) {
    handle_message(message);
  });
}

void TextInput::handle_key_event(uint32_t keycode, bool pressed) {}

void TextInput::handle_message(const FlutterPlatformMessage *message) {
  log::dbg("text_input!");
}

} // namespace shell::plugins