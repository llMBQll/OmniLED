#pragma once

#include <time.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct Clock
{
    struct tm date_time;
    bool updated;
} Clock;


Clock* clock_new();
void clock_delete(Clock* clock);
void clock_update(Clock* clock);