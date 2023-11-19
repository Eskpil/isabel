cmake_minimum_required(VERSION 3.10)

pkg_check_modules(AYLIN REQUIRED aylin)
pkg_check_modules(WAYLAND_EGL REQUIRED wayland-egl)
pkg_check_modules(EGL REQUIRED egl)
pkg_check_modules(GL REQUIRED gl)

set(SOURCES
    "${CMAKE_CURRENT_SOURCE_DIR}/shell/application.cc"
    "${CMAKE_CURRENT_SOURCE_DIR}/shell/window.cc"
    "${CMAKE_CURRENT_SOURCE_DIR}/shell/surface/egl.cc"
    "${CMAKE_CURRENT_SOURCE_DIR}/shell/target.cc"
    "${CMAKE_CURRENT_SOURCE_DIR}/shell/task_runner.cc"
)

set(LIBISABEL "isabel")
add_library(
        ${LIBISABEL}
        SHARED
        ${SOURCES}
)

target_include_directories(
        ${LIBISABEL}
        PUBLIC
        ${CMAKE_CURRENT_SOURCE_DIR}
        ${AYLIN_INCLUDE_DIRS}
        ${WAYLAND_EGL_SOURCE_DIRS}
        ${EGL_INCLUDE_DIRS}
)

set(FLUTTER_EMBEDDER_LIBRARIES "${CMAKE_CURRENT_SOURCE_DIR}/build/libflutter_engine.so")
target_link_libraries(
        ${LIBISABEL}
        PRIVATE
        ${AYLIN_LIBRARIES}
        ${WAYLAND_EGL_LIBRARIES}
        ${FLUTTER_EMBEDDER_LIBRARIES}
        ${EGL_LIBRARIES}
        ${GL_LIBRARIES}
)
