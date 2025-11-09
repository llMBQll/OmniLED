load_app {
    path = get_default_path('clock'),
    args = { '--address', SERVER.Address },
}

load_app {
    path = get_default_path('audio'),
    args = { '--address', SERVER.Address },
}

load_app {
    path = get_default_path('media'),
    args = {
        '--address', SERVER.Address,
        '--mode', 'individual',
        '--map', 'Spotify.exe=SPOTIFY',
    },
}

load_app {
    path = get_default_path('weather'),
    args = {
        '--address', SERVER.Address,
        '--interval', 10,
        '--wind-speed-unit', 'm/s',
        '--temperature-unit', 'Celsius',
        'in', 'Warsaw',
        '--country-code', 'PL',
        '--administrative', 'Masovian',
    }
}