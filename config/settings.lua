Settings {
    animation_ticks_delay = 8,
    animation_ticks_rate = 2,
    font = FontSelector.Default,
    log_level = LevelFilter.Debug,
    keyboard_ticks_repeat_delay = 2,
    keyboard_ticks_repeat_rate = 2,
    steelseries_api = {
        enabled = PLATFORM.Os == 'windows',
        register_heartbeat = true,
        deinitialize_timeout = Duration.from_secs(15),
    },
    update_interval = Duration.from_millis(100),
}
