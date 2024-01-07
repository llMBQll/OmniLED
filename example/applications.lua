local function get_path(application)
    local executable = application .. PLATFORM.ExeSuffix
    local parts = {PLATFORM.ApplicationsDir, executable}
    return table.concat(parts, PLATFORM.PathSeparator)
end

load_app {
    path = get_path('clock'),
    args = { SERVER.address },
}

load_app {
    path = get_path('audio'),
    args = { SERVER.address },
}

load_app {
    path = get_path('media'),
    args = {
        '--address', SERVER.address,
        '--mode', 'individual',
        '--map', 'Spotify.exe=SPOTIFY',
    },
}

load_app {
    path = get_path('weather'),
    args = {
        '--address', SERVER.address,
        '--interval', '10',
        'in', 'Katowice',
    }
}