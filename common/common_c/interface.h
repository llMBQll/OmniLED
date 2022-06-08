#pragma once

#include "managed_string.h"

#define STATUS_OK 0
#define STATUS_ERROR 1

typedef int32_t StatusCode;
typedef void Context;

__declspec(dllexport) StatusCode initialize_impl(Context**);
__declspec(dllexport) StatusCode display_name_impl(Context*, ManagedString*);
__declspec(dllexport) StatusCode types_impl(Context*, ManagedString*);
__declspec(dllexport) StatusCode update_impl(Context*, ManagedString*);
__declspec(dllexport) StatusCode finalize_impl(Context*);