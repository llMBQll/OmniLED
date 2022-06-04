#pragma once

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void(*deleter_t)(char*, size_t);

typedef struct ManagedString
{
    char* str;
    size_t len;
    deleter_t del;
} ManagedString;

void default_deleter(char* string, size_t _length)
{
    printf("[C] Deleter called on %p\n", string);
    free(string);
}

ManagedString managed_string_new()
{
    ManagedString s = {
        .str = NULL,
        .len = 0,
        .del = default_deleter
    };
    return s;
}

ManagedString managed_string_from(const char* string, size_t length)
{
    char* str = (char*)malloc(length + 1);
    strcpy_s(str, length + 1, string);

    ManagedString s = {
        .str = str,
        .len = length,
        .del = default_deleter
    };
    return s;
}

void managed_string_delete(ManagedString* string)
{
    string->del(string->str, string->len);
}
