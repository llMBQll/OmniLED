project(steelseries_oled)
cmake_minimum_required(VERSION 3.10)

add_library(clock_impl STATIC src/interface.c ../../common/common_c/interface.h)
target_include_directories(clock_impl PRIVATE "../../common/common_c")

install(TARGETS clock_impl DESTINATION .)