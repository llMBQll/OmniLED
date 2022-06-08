#include "clock.h"

#include <time.h>
#include <stdlib.h>
#include <string.h>

Clock* clock_new()
{
    Clock* clock = malloc(sizeof(Clock));
    if (!clock)
        return NULL;
    memset(clock, 0, sizeof(Clock));
    clock->date_time.tm_sec = -1; // force update on the first call
    return clock;
}

void clock_delete(Clock* clock)
{
    free(clock);
}

void clock_update(Clock* clock)
{
    time_t raw_time;
    struct tm current;

    time(&raw_time);
    localtime_s(&current, &raw_time);

    clock->updated = current.tm_sec != clock->date_time.tm_sec;
    clock->date_time = current;
}