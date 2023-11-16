cmake_minimum_required(VERSION 3.10)
project(Isabel LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 26)

find_package(PkgConfig)

#add_compile_options(-fsanitize=address)
#add_link_options(-fsanitize=address)

include(shell/build.cmake)
include(examples/build.cmake)