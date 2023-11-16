#include <cstring>

#include "shell/target.h"
#include "shell/log.h"

namespace shell {

Target::Target() {
  log::info("creating base target");
  FlutterRendererConfig config = {};

  config.type = kOpenGL;
  config.open_gl.struct_size = sizeof(config.open_gl);
  config.open_gl.make_current = [](void* data) -> bool {
    auto target = reinterpret_cast<Target *>(data);
    target->m_egl_surface->make_current();
    return true;
  };
  config.open_gl.clear_current = [](void* data) -> bool {
    auto target = reinterpret_cast<Target *>(data);
    target->m_egl_surface->clear_current();
    return true;
  };
  config.open_gl.present = [](void* data) -> bool {
    auto target = reinterpret_cast<Target *>(data);
    target->m_egl_surface->swap_buffers();
    return true;
  };
  config.open_gl.fbo_callback = [](void*) -> uint32_t {
    return 0;  // FBO0
  };
  config.open_gl.gl_proc_resolver = [](void* data, const char* name) -> void* {
    auto target = reinterpret_cast<Target *>(data);
    return target->m_egl_surface->get_proc_address(name);
  };

  FlutterProjectArgs args = {
      .struct_size = sizeof(FlutterProjectArgs),
      .assets_path = "/home/beta/Repos/github.com/flutter-engine/src/flutter/examples/glfw/debug/myapp/build/flutter_assets/",
      .icu_data_path = "/home/beta/Repos/github.com/flutter-engine/src/out/host_debug_unopt/icudtl.dat",  // Find this in your bin/cache directory.
  };

  log::info("assets_path: ({})", args.assets_path);
  log::info("icu_data_path: ({})", args.icu_data_path);

  FlutterEngineResult result = FlutterEngineInitialize(FLUTTER_ENGINE_VERSION, &config, &args, this, &m_engine);
  if (result != kSuccess || m_engine == nullptr) {
    log::fatal("could not initialize the flutter engine");
  }
}

void Target::run() {
  FlutterEngineResult result = FlutterEngineRunInitialized(m_engine);
  if (result != kSuccess) {
    log::fatal("could not run the flutter engine");
  }

  m_engine_is_running = true;
  update_window_metrics(1, 1, 1);
}

void Target::update_window_metrics(int width, int height, int pixel_ratio) {
  if (!m_engine_is_running)
    return;

  m_egl_target->resize_onscreen_window(width, height);

  FlutterWindowMetricsEvent event = {};
  event.struct_size = sizeof(event);
  event.width = width;
  event.height = height;
  event.pixel_ratio = pixel_ratio;

  log::dbg("sending new window metrics ({}x{})", width, height);

  FlutterEngineSendWindowMetricsEvent(
      m_engine,
      &event);
}

}
