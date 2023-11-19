#include <chrono>

#include "shell/log.h"
#include "shell/target.h"

#include "shell/plugins/text_input/text_input.h"

namespace shell {

Target::Target(std::string_view flutter_assets, std::string_view icudtl_dat) {
  FlutterRendererConfig config = {};

  m_task_runner = std::make_unique<TaskRunner>(
      FlutterEngineGetCurrentTime, [this](const auto *task) {
        if (!m_engine_is_running)
          return;

        FlutterEngineRunTask(m_engine, task);
      });

  m_dispatcher = std::make_shared<Dispatcher>();
  m_plugins = std::make_unique<PluginManager>();

  config.type = kOpenGL;
  config.open_gl.struct_size = sizeof(config.open_gl);

  config.open_gl.make_resource_current = [](void *data) -> bool {
    auto target = reinterpret_cast<Target *>(data);
    target->m_resource_surface->make_current();
    return true;
  };

  config.open_gl.make_current = [](void *data) -> bool {
    auto target = reinterpret_cast<Target *>(data);
    target->m_primary_surface->make_current();
    return true;
  };
  config.open_gl.clear_current = [](void *data) -> bool {
    auto target = reinterpret_cast<Target *>(data);
    target->m_primary_surface->clear_current();
    return true;
  };
  config.open_gl.present = [](void *data) -> bool {
    auto target = reinterpret_cast<Target *>(data);
    target->m_primary_surface->swap_buffers();
    return true;
  };
  config.open_gl.fbo_callback = [](void *) -> uint32_t {
    return 0; // FBO0
  };
  config.open_gl.gl_proc_resolver = [](void *data, const char *name) -> void * {
    auto target = reinterpret_cast<Target *>(data);
    return target->m_primary_surface->get_proc_address(name);
  };

  // Configure task runners.
  FlutterTaskRunnerDescription platform_task_runner = {};
  platform_task_runner.struct_size = sizeof(FlutterTaskRunnerDescription);
  platform_task_runner.user_data = m_task_runner.get();
  platform_task_runner.runs_task_on_current_thread_callback =
      [](void *data) -> bool {
    auto task_runner = reinterpret_cast<TaskRunner *>(data);
    return task_runner->runs_tasks_on_current_thread();
  };

  platform_task_runner.post_task_callback =
      [](FlutterTask task, uint64_t target_time_nanos, void *data) -> void {
    auto task_runner = reinterpret_cast<TaskRunner *>(data);
    task_runner->post_delayed_task(task, target_time_nanos);
  };

  FlutterCustomTaskRunners custom_task_runners = {};
  custom_task_runners.struct_size = sizeof(FlutterCustomTaskRunners);
  custom_task_runners.platform_task_runner = &platform_task_runner;

  FlutterProjectArgs args = {
      .struct_size = sizeof(FlutterProjectArgs),
      .assets_path = flutter_assets.data(),
      .icu_data_path = icudtl_dat.data(),

      .platform_message_callback =
          [](const FlutterPlatformMessage *message, void *data) {
            auto target = reinterpret_cast<Target *>(data);
            target->m_dispatcher->dispatch(message);
          },

      .custom_task_runners = &custom_task_runners,
  };

  FlutterEngineResult result = FlutterEngineInitialize(
      FLUTTER_ENGINE_VERSION, &config, &args, this, &m_engine);
  if (result != kSuccess || m_engine == nullptr) {
    log::fatal("could not initialize the flutter engine");
  }

  create_platform_listeners();
}

void Target::create_platform_listeners() {
  m_plugins->create<plugins::TextInput>(m_dispatcher);
}

void Target::run() {
  FlutterEngineResult result = FlutterEngineRunInitialized(m_engine);
  if (result != kSuccess) {
    log::fatal("could not run the flutter engine");
  }

  m_engine_is_running = true;
  update_window_metrics(1, 1, 1);
}

// TODO: IMPORTANT handle serial.
void Target::handle_key_pressed(uint32_t keycode, bool pressed, uint32_t serial,
                                size_t timestamp) {
  if (!m_engine_is_running)
    return;

  auto text_input = m_plugins->get<plugins::TextInput>("text_input");
  text_input->handle_key_event(keycode, pressed);
}

void Target::handle_pointer_motion(double x, double y, size_t timestamp) {
  if (!m_engine_is_running)
    return;

  // NOTE: the hover event can not be sent if the flutter pointer is pressed.
  // When a
  //       pointer button is pressed under a motion event flutter expects the
  //       phase to be set as move.
  auto phase = FlutterPointerPhase::kHover;
  if (m_last_pointer_phase == FlutterPointerPhase::kDown) {
    phase = FlutterPointerPhase::kMove;
  }

  FlutterPointerEvent event = {
      .struct_size = sizeof(FlutterPointerEvent),
      .phase = phase,
      .timestamp = timestamp,
      .x = x,
      .y = y,
      .device = 69,
      .device_kind = FlutterPointerDeviceKind::kFlutterPointerDeviceKindMouse,
  };

  FlutterEngineSendPointerEvent(m_engine, &event, 1);
}

void Target::handle_pointer_enter(double x, double y) {
  if (!m_engine_is_running)
    return;

  auto timestamp =
      std::chrono::duration_cast<std::chrono::microseconds>(
          std::chrono::high_resolution_clock::now().time_since_epoch())
          .count();

  FlutterPointerEvent event = {
      .struct_size = sizeof(FlutterPointerEvent),
      .phase = FlutterPointerPhase::kAdd,
      .timestamp = static_cast<size_t>(timestamp),
      .x = x,
      .y = y,
      .device = 69,
      .device_kind = FlutterPointerDeviceKind::kFlutterPointerDeviceKindMouse,
  };

  FlutterEngineSendPointerEvent(m_engine, &event, 1);
}

void Target::handle_pointer_leave() {
  if (!m_engine_is_running)
    return;

  auto timestamp =
      std::chrono::duration_cast<std::chrono::microseconds>(
          std::chrono::high_resolution_clock::now().time_since_epoch())
          .count();

  FlutterPointerEvent event = {
      .struct_size = sizeof(FlutterPointerEvent),
      .phase = FlutterPointerPhase::kRemove,
      .timestamp = static_cast<size_t>(timestamp),
      .device = 69,
      .device_kind = FlutterPointerDeviceKind::kFlutterPointerDeviceKindMouse,
  };

  FlutterEngineSendPointerEvent(m_engine, &event, 1);
}

void Target::handle_pointer_button(double x, double y, bool pressed,
                                   FlutterPointerMouseButtons button,
                                   size_t timestamp) {
  if (!m_engine_is_running)
    return;

  FlutterPointerEvent event = {
      .struct_size = sizeof(FlutterPointerEvent),
      .phase = pressed ? FlutterPointerPhase::kDown : FlutterPointerPhase::kUp,
      .timestamp = timestamp,
      .x = x,
      .y = y,
      .device = 69,
      .device_kind = FlutterPointerDeviceKind::kFlutterPointerDeviceKindMouse,
      .buttons = button,
  };
  m_last_pointer_phase = event.phase;
  m_last_pointer_button = button;

  FlutterEngineSendPointerEvent(m_engine, &event, 1);
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

  FlutterEngineSendWindowMetricsEvent(m_engine, &event);
}

void Target::perform_tasks() { m_task_runner->process(); }

} // namespace shell
