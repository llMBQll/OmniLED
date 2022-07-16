#include "interface.h"

#include <time.h>
#include <stdio.h>
#include <Windows.h>

#define BUF_SIZE 127

void send_update(OnUpdateCallbackFn on_update, struct tm* time);

StatusCode run_impl(const int32_t* keep_running, OnUpdateCallbackFn on_update)
{
    struct tm last_update;
    struct tm current;
    time_t raw_time;

    last_update.tm_sec = -1;
    while (*keep_running)
    {
        time(&raw_time);
        localtime_s(&current, &raw_time);
        if (last_update.tm_sec != current.tm_sec)
        {
            last_update = current;
            send_update(on_update, &current);
        }
        Sleep(50);
    }

    return STATUS_OK;
}

void send_update(OnUpdateCallbackFn on_update, struct tm* time)
{
    char buf[BUF_SIZE + 1];

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
        return;
    on_update(buf, (uint32_t)n);
}