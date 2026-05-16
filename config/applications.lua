load_plugin {
    path = get_default_plugin_path('clock'),
    args = { },
}

-- load_app {
--     path = get_default_path('audio'),
--     args = { '--address', SERVER.Address },
-- }
--
-- load_app {
--     path = get_default_path('media'),
--     args = {
--         '--address', SERVER.Address,
--         '--mode', 'individual',
--         '--map', 'Spotify.exe=SPOTIFY',
--     },
-- }

load_plugin {
    path = get_default_plugin_path('system'),
    args = { '--interval', '2sec' },
}

-- load_app {
--     path = get_default_path('weather'),
--     args = {
--         '--address', SERVER.Address,
--         '--interval', '10min',
--         '--wind-speed-unit', 'm/s',
--         '--temperature-unit', 'Celsius',
--         'in', 'Warsaw',
--         '--country-code', 'PL',
--         '--administrative', 'Mazovia',
--     }
-- }
