#include "clock.h"
#include "interface.h"

#define BUF_SIZE 127

StatusCode initialize_impl(Context** ctx)
{
    Clock* clock = clock_new();
    if (!clock)
        return STATUS_ERROR;
    *ctx = (Context*) clock;
    return STATUS_OK;
}

StatusCode display_name_impl(Context* ctx, ManagedString* string)
{
    const char* display_name = "Clock";

    *string = managed_string_from_static(display_name, strlen(display_name));
    return STATUS_OK;
}

StatusCode types_impl(Context* ctx, ManagedString* json)
{
    const char* types =
        "{"
            "\"Seconds\":\"number\","
            "\"Minutes\":\"number\","
            "\"Hours\":\"number\","
            "\"Month Day\":\"number\","
            "\"Month\":\"number\","
            "\"Year\":\"number\","
            "\"Week Day\":\"number\""
        "}";

    *json = managed_string_from_static(types, strlen(types));
    return STATUS_OK;
}

StatusCode update_impl(Context* ctx, ManagedString* json)
{
    char buf[BUF_SIZE + 1];

    Clock* clock = (Clock*) ctx;
    clock_update(clock);
    if (!clock->updated)
        return STATUS_OK;

    struct tm* time = &clock->date_time;
    int n = snprintf(buf, BUF_SIZE,
        "{"
            "\"Seconds\":\"%d\","
            "\"Minutes\":\"%d\","
            "\"Hours\":\"%d\","
            "\"Month Day\":\"%d\","
            "\"Month\":\"%d\","
            "\"Year\":\"%d\","
            "\"Week Day\":\"%d\""
        "}",
        time->tm_sec,
        time->tm_min,
        time->tm_hour,
        time->tm_mday,
        time->tm_mon + 1,                      // [0, 11] -> [1, 12]
        time->tm_year + 1900,                  // years since 1900 -> current year
        time->tm_wday == 0 ? 7 : time->tm_wday // [0, 6] -> [1, 7] - move Sunday from 0 to 7
    );
    if (n < 0)
        return STATUS_ERROR;

    *json = managed_string_copy_temp(buf, (size_t) n);
    return STATUS_OK;
}

StatusCode finalize_impl(Context* ctx)
{
    Clock* clock = (Clock*) ctx;
    clock_delete(clock);
    return STATUS_OK;
}