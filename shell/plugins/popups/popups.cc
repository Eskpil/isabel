#include <memory>

#include "shell/plugins/popups/args.h"
#include "shell/plugins/popups/popups.h"

namespace shell::plugins {

static const std::string_view k_channel = "isabel.io/popups";

Popups::Popups(std::shared_ptr<Dispatcher> dis) : Plugin(dis) {
  dispatcher()->on(k_channel, [this](const FlutterPlatformMessage *message) {
    handle_message(message);
  });
}

void Popups::handle_message(const FlutterPlatformMessage *message) {
  auto method_call = MethodCall(
      reinterpret_cast<const char *>(message->message), message->message_size);

  if (method_call.method() == "spawn_popup") {
    auto args = method_call.args<SpawnPopupArgs>();
    log::dbg("spawn_popup@{}x{}", args->x, args->y);
  }
}
} // namespace shell::plugins
