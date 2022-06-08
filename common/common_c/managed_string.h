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

static void default_deleter(char* string, size_t _length)
{
    free(string);
}

static void static_deleter(char* string, size_t _length)
{

}

static ManagedString managed_string_new()
{
    ManagedString s = {
        .str = NULL,
        .len = 0,
        .del = default_deleter
    };
    return s;
}

static ManagedString managed_string_copy_temp(const char* string, size_t length)
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

static ManagedString managed_string_move_temp(char* string, size_t length)
{
    ManagedString s = {
        .str = string,
        .len = length,
        .del = default_deleter
    };
    return s;
}

static ManagedString managed_string_move_temp_with_deleter(char* string, size_t length, deleter_t deleter)
{
    ManagedString s = {
        .str = string,
        .len = length,
        .del = deleter
    };
    return s;
}

static ManagedString managed_string_from_static(const char* string, size_t length)
{
    ManagedString s = {
        .str = (char*)string,
        .len = length,
        .del = static_deleter
    };
    return s;
}

static void managed_string_delete(ManagedString* string)
{
    string->del(string->str, string->len);
}
