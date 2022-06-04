project(c_test_impl)
cmake_minimum_required(VERSION 3.10)

add_library(c_test_impl STATIC src/lib.c)
target_include_directories(c_test_impl PRIVATE "../../common/common_c")

install(TARGETS c_test_impl DESTINATION .)