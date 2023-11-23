#include <cstring>

#include "shell/log.h"
#include "shell/window.h"

#include <GL/gl.h>

namespace shell {

static std::unique_ptr<surface::EglTarget>
create_egl_target(struct aylin_shell *shell, int width, int height) {
  auto target = std::make_unique<surface::EglTarget>(shell->app->display);

  auto onscreen_window =
      wl_egl_window_create(aylin_shell_get_surface(shell), width, height);
  if (onscreen_window == nullptr) {
    log::fatal("could not create onscreen window: ({})", strerror(errno));
  }
  target->set_onscreen_window(
      reinterpret_cast<EGLNativeWindowType>(onscreen_window));

  auto offscreen_window = wl_egl_window_create(
      aylin_application_create_independent_surface(shell->app), 1, 1);
  if (offscreen_window == nullptr) {
    log::fatal("could not create offscreen window: ({})", strerror(errno));
  }
  target->set_offscreen_window(
      reinterpret_cast<EGLNativeWindowType>(offscreen_window));

  return target;
}

const struct aylin_shell_listener Window::k_window_listener = {
    .close =
        [](struct aylin_shell *shell, void *userdata) {
          // TODO: Shutdown the flutter engine and destroy all related egl and
          // wayland resources.
          exit(1);
        },
    .resize =
        [](struct aylin_shell *shell, struct aylin_shell_resize_event *event,
           void *userdata) {
          auto window = reinterpret_cast<Window *>(userdata);

          window->m_height = event->height;
          window->m_width = event->width;

          // TODO: Figure out what to do with pixel ratio.
          window->update_window_metrics(event->width, event->height, 1);
        },
    .pointer_enter =
        [](struct aylin_shell *shell,
           struct aylin_shell_pointer_enter_event *event, void *userdata) {
          auto window = reinterpret_cast<Window *>(userdata);
          window->handle_pointer_enter(event->x, event->y);
        },
    .pointer_leave =
        [](struct aylin_shell *shell,
           struct aylin_shell_pointer_leave_event *event, void *userdata) {
          auto window = reinterpret_cast<Window *>(userdata);
          window->handle_pointer_leave();
        },
    .pointer_button =
        [](struct aylin_shell *shell,
           struct aylin_shell_pointer_button_event *event, void *userdata) {
          auto window = reinterpret_cast<Window *>(userdata);

          auto button =
              FlutterPointerMouseButtons::kFlutterPointerButtonMousePrimary;
          switch (event->button) {
          case left:
            break;
          case right:
            button =
                FlutterPointerMouseButtons::kFlutterPointerButtonMouseSecondary;
            break;
          case middle:
            button =
                FlutterPointerMouseButtons::kFlutterPointerButtonMouseMiddle;
            break;
          case back:
            button = FlutterPointerMouseButtons::kFlutterPointerButtonMouseBack;
            break;
          default:
            return;
          }

          window->handle_pointer_button(
              event->x, event->y, event->action == press, button,
              static_cast<size_t>(event->timestamp), event->serial);
        },
    .pointer_motion =
        [](struct aylin_shell *shell,
           struct aylin_shell_pointer_motion_event *event, void *userdata) {
          auto window = reinterpret_cast<Window *>(userdata);
          window->handle_pointer_motion(event->x, event->y,
                                        static_cast<size_t>(event->timestamp),
                                        event->serial);
        },
    .key_pressed =
        [](struct aylin_shell *shell,
           struct aylin_shell_key_pressed_event *event, void *userdata) {
          auto window = reinterpret_cast<Window *>(userdata);
          window->handle_key_pressed(event->keycode, event->symbol,
                                     event->action == press, event->serial,
                                     static_cast<size_t>(event->timestamp));
        },
    .frame = [](struct aylin_shell *shell,
                struct aylin_shell_frame_event *event, void *userdata) {},
};

Window::Window(shell::WindowOptions options)
    : Target(options.flutter_assets_path, options.icudtl_path),
      m_options(options) {}

void Window::configure(shell::Application *app) {
  m_application = app;
  m_configured = true;

  m_width = m_options.width;
  m_height = m_options.height;

  // create shell
  m_shell = aylin_window_create(app->inner(), &Window::k_window_listener, this);
  if (m_shell == nullptr) {
    log::fatal("could not create window: ({})", strerror(errno));
  }

  m_egl_target = create_egl_target(m_shell, m_width, m_height);
  m_primary_surface = m_egl_target->create_onscreen_surface();
  m_resource_surface = m_egl_target->create_offscreen_surface();

  aylin_window_set_title(m_shell, const_cast<char *>(m_options.title.data()));
}

void Window::set_title(std::string_view title) {
  m_options.title = title;

  if (m_configured) {
    aylin_window_set_title(m_shell, const_cast<char *>(title.data()));
  }
}

void Window::initiate_move(uint32_t serial) {
  aylin_window_move(m_shell, serial);
}

int Window::width() { return m_width; }

int Window::height() { return m_height; }

} // namespace shell
