#include "shell/dispatcher.h"
#include "shell/log.h"

#include "shell/plugins/plugin.h"

namespace shell {
Dispatcher::Dispatcher(Dispatcher::EngineCallback engine_callback)
    : m_engine_callback(std::move(engine_callback)) {
  m_listeners = std::unordered_map<std::string_view, Entry>();
}

void Dispatcher::on(std::string_view channel,
                    shell::Dispatcher::DispatchCallback callback) {
  auto entry = Entry{
      .channel = channel,
      .callback = callback,
  };

  m_listeners.emplace(channel, entry);
}

void Dispatcher::dispatch(const FlutterPlatformMessage *message) {
  auto channel = std::string_view(message->channel);

  // no handlers
  if (!m_listeners.count(channel))
    return;

  auto entry = m_listeners[channel];
  entry.callback(message);
}

void Dispatcher::call(std::string_view channel, const char *bytes,
                      size_t size) {
  FlutterPlatformMessage message = {
      .struct_size = sizeof(FlutterPlatformMessage),
      .channel = channel.data(),
      .message = reinterpret_cast<const uint8_t *>(bytes),
      .message_size = size,
  };

  m_engine_callback(&message);
}

void Dispatcher::call(std::string_view channel,
                      std::unique_ptr<plugins::MethodCall> method_call) {
  auto buffer = method_call->build();

  FlutterPlatformMessage message = {
      .struct_size = sizeof(FlutterPlatformMessage),
      .channel = channel.data(),
      .message = reinterpret_cast<const uint8_t *>(buffer),
      .message_size = strlen(buffer),
  };

  m_engine_callback(&message);
}

Dispatcher::~Dispatcher() {}
} // namespace shell