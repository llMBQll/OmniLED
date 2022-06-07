#include "clock.h"
#include "managed_string.h"

#define DLL_EXPORT __declspec(dllexport)
#define BUF_SIZE 63

typedef void Context;

//char buf[BUF_SIZE];

DLL_EXPORT int initialize_impl(Context** ctx, ManagedString* application_name)
{
    Clock* clock = clock_new();
    if (!clock)
        return 1;
    *ctx = (Context*)clock;
    return 0;
}

DLL_EXPORT int display_name_impl(Context* ctx, ManagedString* string)
{
    const char* display_name = "Clock";
    printf("[0x%p] - C static alloc\n", display_name);
    fflush(stdout);

    Clock* clock = (Clock*) ctx;
    *string = managed_string_from_static(display_name, strlen(display_name));
    return 0;
}

DLL_EXPORT int types_impl(Context* ctx, ManagedString* json)
{
    const char* types = "[{\"timestamp\":\"number\"}]";
    printf("[0x%p] - C static alloc\n", types);
    fflush(stdout);

//    Clock* clock = (Clock*) ctx;
    *json = managed_string_from_static(types, strlen(types));
    return 0;
}

DLL_EXPORT int update_impl(Context* ctx, ManagedString* json)
{
    char buf[BUF_SIZE + 1];

    Clock* clock = (Clock*) ctx;
    clock_update(clock);
    if (!clock->updated)
    {
        const char* tmp = "";
        *json = managed_string_from_static(tmp, 0);
        printf("[0x%p] - C static alloc\n", tmp);
        fflush(stdout);
        return 0;
    }

    int n = snprintf(buf, BUF_SIZE, "[{\"timestamp\":%llu}]", clock->timestamp);
    if (n < 0)
        return -1;

    *json = managed_string_copy_temp(buf, (size_t) n);
    return 0;
}

DLL_EXPORT int finalize_impl(Context* ctx)
{
    if (!ctx)
        return 1;
    Clock* clock = (Clock*)ctx;
    clock_delete(clock);
    return 0;
}