#include "clock.h"

#include <time.h>
#include <stdlib.h>

Clock* clock_new()
{
    Clock* clock = malloc(sizeof(Clock));
    if (!clock)
        return NULL;

    clock->timestamp = 0;
    clock->updated = false;

    return clock;
}

void clock_delete(Clock* clock)
{
    free(clock);
}

void clock_update(Clock* clock)
{
    uint64_t last = clock->timestamp;
    uint64_t current = time(NULL);

    clock->updated = last != current;
    if (clock->updated)
        clock->timestamp = current;
}
