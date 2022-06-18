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

StatusCode name_impl(Context* ctx, ManagedString* string)
{
    const char* display_name = "CLOCK";

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
            "\"MonthDay\":\"number\","
            "\"Month\":\"number\","
            "\"Year\":\"number\","
            "\"WeekDay\":\"number\""
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
            "\"Seconds\":%d,"
            "\"Minutes\":%d,"
            "\"Hours\":%d,"
            "\"MonthDay\":%d,"
            "\"Month\":%d,"
            "\"Year\":%d,"
            "\"WeekDay\":%d"
        "}",
        time->tm_sec,
        time->tm_min,
        time->tm_hour,
        time->tm_mday,
        time->tm_mon,
        time->tm_year + 1900,                      // years since 1900 -> current year
        time->tm_wday == 0 ? 6 : time->tm_wday - 1 // [Sun, Mon, Tue, Wed, Thu, Fri, Sat] -> [Mon, Tue, Wed, Thu, Fri, Sat, Sun]
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