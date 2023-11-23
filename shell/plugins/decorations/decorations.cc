#include "shell/plugins/decorations/decorations.h"
#include "shell/plugins/decorations/args.h"

#include "shell/target.h"

namespace shell::plugins {

static const std::string_view k_channel = "isabel.io/decorations";

Decorations::Decorations(std::shared_ptr<Dispatcher> dis, shell::Target *target)
    : Plugin(dis), m_target(target) {
  dispatcher()->on(k_channel, [this](const FlutterPlatformMessage *message) {
    handle_message(message);
  });
}

void Decorations::handle_message(const FlutterPlatformMessage *message) {
  auto method_call = MethodCall(
      reinterpret_cast<const char *>(message->message), message->message_size);

  if (method_call.method() == "initiate_move") {
    auto args = method_call.args<InitiateMoveArgs>();
    m_target->initiate_move(args->serial);
  }
}

void Decorations::move(uint32_t serial) {
  auto args = std::make_shared<PointerHoldEventArgs>();

  args->serial = serial;

  dispatcher()->call(k_channel, std::make_unique<MethodCall>("move", args));
}
} // namespace shell::plugins