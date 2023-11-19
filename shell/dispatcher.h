#ifndef ISABEL_DISPATCHER_H
#define ISABEL_DISPATCHER_H

#include <functional>
#include <map>
#include <memory>
#include <string_view>

#include "shell/flutter/embedder.h"

namespace shell {

namespace plugins {
class MethodCall;
}

class Dispatcher {
  friend class Target;

public:
  using EngineCallback = std::function<void(const FlutterPlatformMessage *)>;
  using DispatchCallback = std::function<void(const FlutterPlatformMessage *)>;

  explicit Dispatcher(EngineCallback);
  ~Dispatcher();

  void on(std::string_view, DispatchCallback);
  void call(std::string_view, std::unique_ptr<plugins::MethodCall>);
  void call(std::string_view, const char *, size_t);

private:
  void dispatch(const FlutterPlatformMessage *);

  struct Entry {
    std::string_view channel;
    DispatchCallback callback;
  };

  std::unordered_map<std::string_view, Entry> m_listeners;

  EngineCallback m_engine_callback;
};
} // namespace shell

#endif // ISABEL_DISPATCHER_H
