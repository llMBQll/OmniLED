local function get_app_path(name)
    return '..' .. PLATFORM.PathSeparator .. 'target' .. PLATFORM.PathSeparator .. 'debug'
                .. PLATFORM.PathSeparator .. name .. PLATFORM.ExeSuffix
end

load_app {
    path = get_app_path('clock'),
    args = { '--address', SERVER.Address },
}
