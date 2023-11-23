#ifndef ISABEL_SHELL_PLUGINS_DECORATIONS_ARGS_H
#define ISABEL_SHELL_PLUGINS_DECORATIONS_ARGS_H

#include "shell/plugins/plugin.h"

namespace shell::plugins {
class PointerHoldEventArgs : public MethodArguments {
public:
  void serialize(struct json_object *object) {
    {
      auto serial_obj = json_object_new_uint64(serial);
      json_object_object_add(object, "serial", serial_obj);
    }
  }

  enum json_type type() { return json_type_object; }

  uint32_t serial;
};

class InitiateMoveArgs : public MethodArguments {
public:
  void deserialize(struct json_object *object) {
    {
      auto serial_obj = json_object_object_get(object, "serial");
      serial = json_object_get_uint64(serial_obj);
    }
  }

  uint32_t serial;
};
} // namespace shell::plugins

#endif // ISABEL_SHELL_PLUGINS_DECORATIONS_ARGS_H