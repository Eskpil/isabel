#include "shell/plugins/text_input/text_input.h"
#include "shell/log.h"
#include "shell/plugins/text_input/args.h"

namespace shell::plugins {

static const auto k_channel = "flutter/textinput";

static const std::string_view k_set_client_method = "TextInput.setClient";

TextInput::TextInput(std::shared_ptr<Dispatcher> dispatch) : Plugin(dispatch) {
  dispatcher()->on(k_channel, [this](const FlutterPlatformMessage *message) {
    handle_message(message);
  });
}

void TextInput::handle_key_event(uint32_t keycode, bool pressed) {}

void TextInput::handle_message(const FlutterPlatformMessage *message) {
  auto method_call = MethodCall(
      reinterpret_cast<const char *>(message->message), message->message_size);

  if (method_call.method() == k_set_client_method) {
    auto args = method_call.args<SetClientArgs>();

    log::dbg("setClient {} {}", args->input_type, args->input_action);
  }
}

} // namespace shell::plugins