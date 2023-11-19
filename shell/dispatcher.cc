#include "shell/dispatcher.h"
#include "shell/log.h"

namespace shell {
Dispatcher::Dispatcher() {
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

Dispatcher::~Dispatcher() {}
} // namespace shell