#pragma once

#include <stdint.h>

#define STATUS_OK 0
#define STATUS_ERROR 1

typedef int32_t StatusCode;
typedef StatusCode(*OnUpdateCallbackFn)(const char*, uint32_t);
typedef StatusCode(*RunFn)(OnUpdateCallbackFn);

__declspec(dllexport) StatusCode run_impl(const int32_t*, OnUpdateCallbackFn);