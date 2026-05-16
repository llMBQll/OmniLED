#ifndef MBQ_OMNI_LED_API_H
#define MBQ_OMNI_LED_API_H

typedef enum LogLevel {
    LOG_LEVEL_ERROR = 0,
    LOG_LEVEL_WARN = 1,
    LOG_LEVEL_INFO = 2,
    LOG_LEVEL_DEBUG = 3,
    LOG_LEVEL_TRACE = 4,
} LogLevel;

typedef void(*omni_led_event_t)(
    const unsigned char* event_data,
    unsigned long long event_data_length
);

typedef void(*omni_led_log_t)(
    LogLevel level,
    const char* target,
    unsigned long long target_length,
    const char* message,
    unsigned long long message_length
);

typedef struct OmniLedApi {
    omni_led_event_t event;
    omni_led_log_t log;
} OmniLedApi;

typedef int(*omni_led_run_t)(
    OmniLedApi api,
    int argc,
    char** argv
);

#ifdef _WIN32
    #ifdef MBQ_OMNI_LED_HOST
        #define MBQ_OMNI_LED_EXPORTED __declspec(dllimport)
    #else
        #define MBQ_OMNI_LED_EXPORTED __declspec(dllexport)
    #endif // MBQ_OMNI_LED_HOST
#else
     #define MBQ_OMNI_LED_EXPORTED
#endif // _WIN32

// Plugin entry point
MBQ_OMNI_LED_EXPORTED int omni_led_run(OmniLedApi api, int argc, char** argv);

#endif // MBQ_OMNI_LED_API_H
