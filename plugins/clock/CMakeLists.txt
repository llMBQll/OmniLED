project(clock_impl)
cmake_minimum_required(VERSION 3.10)

add_library(clock_impl STATIC src/lib.c src/clock.c src/clock.h ../../common/common_c/managed_string.h)
target_include_directories(clock_impl PRIVATE "../../common/common_c")

install(TARGETS clock_impl DESTINATION .)