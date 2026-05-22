load_plugin {
    path = get_default_plugin_path('clock'),
    args = { },
}

load_plugin {
    path = get_default_plugin_path('audio'),
    args = { },
}

load_plugin {
    path = get_default_plugin_path('media'),
    args = {
        '--mode', 'individual',
        '--map', 'Spotify.exe=SPOTIFY',
    },
}

load_plugin {
    path = get_default_plugin_path('system'),
    args = { '--interval', '2sec' },
}

load_plugin {
    path = get_default_plugin_path('weather'),
    args = {
        '--interval', '10min',
        '--wind-speed-unit', 'm/s',
        '--temperature-unit', 'Celsius',
        'in', 'Warsaw',
        '--country-code', 'PL',
        '--administrative', 'Mazovia',
    }
}
