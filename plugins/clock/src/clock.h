#pragma once

#include <stdint.h>
#include <stdbool.h>

typedef struct Clock
{
    uint64_t timestamp;
    bool updated;
} Clock;


Clock* clock_new();
void clock_delete(Clock* clock);
void clock_update(Clock* clock);