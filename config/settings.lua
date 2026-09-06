Settings.animation_ticks_delay = 8
Settings.animation_ticks_rate = 2
Settings.font = FontSelector.Default
Settings.log_filter_map = LogFilterMap.default_with(LevelFilter.Debug)
Settings.keyboard_ticks_repeat_delay = 2
Settings.keyboard_ticks_repeat_rate = 2
Settings.steelseries_api.enabled = PLATFORM.Os ~= 'linux'
Settings.steelseries_api.register_heartbeat = true
Settings.steelseries_api.deinitialize_timeout = Duration.from_secs(15)
Settings.update_interval = Duration.from_millis(100)
