Settings = {
    animation_ticks_delay = 8,
    animation_ticks_rate = 2,
    default_font = FontSelector.Builtin,
    intern_retention_period = Duration.from_secs(10),
    level_filter_table = LevelFilterTable.default_with(LevelFilter.Debug),
    keyboard_ticks_repeat_delay = 2,
    keyboard_ticks_repeat_rate = 2,
    steelseries_api = {
        enabled = PLATFORM.Os ~= 'linux',
        register_heartbeat = true,
        deinitialize_timeout = Duration.from_secs(15),
    },
    update_interval = Duration.from_millis(100),
}
