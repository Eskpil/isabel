set(WINDOW_EXAMPLE "window-example")

add_executable(${WINDOW_EXAMPLE} "${CMAKE_CURRENT_SOURCE_DIR}/examples/window/main.cc")
target_link_libraries(${WINDOW_EXAMPLE} PRIVATE ${LIBISABEL})