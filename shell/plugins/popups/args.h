//
// Created by beta on 11/24/23.
//

#ifndef ISABEL_PLUGINS_POPUPS_ARGS_H
#define ISABEL_PLUGINS_POPUPS_ARGS_H

#include "shell/plugins/plugin.h"

namespace shell::plugins {
class SpawnPopupArgs : public MethodArguments {
public:
  void deserialize(struct json_object *obj) {
    {
      auto x_obj = json_object_object_get(obj, "x");
      x = json_object_get_double(x_obj);
    }
    {
      auto y_obj = json_object_object_get(obj, "y");
      y = json_object_get_double(y_obj);
    }
  }

  double x, y;
};
} // namespace shell::plugins

#endif // ISABEL_PLUGINS_POPUPS_ARGS_H
