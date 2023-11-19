#ifndef ISABEL_DISPATCHER_H
#define ISABEL_DISPATCHER_H

#include <functional>
#include <map>
#include <string_view>

#include "shell/flutter/embedder.h"

namespace shell {
class Dispatcher {
  friend class Target;

public:
  using DispatchCallback = std::function<void(const FlutterPlatformMessage *)>;

  Dispatcher();
  ~Dispatcher();

  void on(std::string_view, DispatchCallback);

private:
  void dispatch(const FlutterPlatformMessage *);

  struct Entry {
    std::string_view channel;
    DispatchCallback callback;
  };

  std::unordered_map<std::string_view, Entry> m_listeners;
};
} // namespace shell

#endif // ISABEL_DISPATCHER_H
