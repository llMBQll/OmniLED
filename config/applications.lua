load_app {
    path = get_default_path('clock'),
    args = { '--address', SERVER.address },
}

load_app {
    path = get_default_path('audio'),
    args = { '--address', SERVER.address },
}

load_app {
    path = get_default_path('media'),
    args = {
        '--address', SERVER.address,
        '--mode', 'individual',
        '--map', 'SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify=SPOTIFY',
    },
}

load_app {
    path = get_default_path('weather'),
    args = {
        '--address', SERVER.address,
        '--interval', '10',
        'in', 'Katowice',
    }
}