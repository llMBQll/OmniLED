#include <stdio.h>
#include "managed_string.h"

#define DLL_EXPORT __declspec(dllexport)

typedef void* Context;

typedef struct Data
{
    const char* tmp;
} Data;

DLL_EXPORT int initialize_impl(Context* ctx)
{
    Data* data = malloc(sizeof(Data));
    data->tmp = "Hello from C plugin";
    *ctx = (Context)data;
    return 0;
}

DLL_EXPORT int update_impl(Context ctx, ManagedString* string)
{
    Data* data = (Data*)ctx;
    *string = managed_string_from(data->tmp, strlen(data->tmp));
    return 0;
}

DLL_EXPORT int finalize_impl(Context ctx)
{
    free(ctx);
    return 0;
}