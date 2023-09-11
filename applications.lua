local function make_path(path)
    if PLATFORM.os == 'windows' then
        APPLICATIONS_PATH = 'C:\\dev\\rust\\steelseries_oled_applications\\target\\release\\'
        EXTENSION = '.exe'

        return APPLICATIONS_PATH .. path .. EXTENSION
    elseif PLATFORM.os == 'linux' then
        APPLICATIONS_PATH = '/home/mbq/dev/rust/steelseries_oled_applications/target/release/'
        EXTENSION = ''

        return APPLICATIONS_PATH .. path .. EXTENSION
    end

    LOG:error('Unexpected platform: ' .. tostring(PLATFORM.os))
end

load_app {
    path = make_path('clock'),
    args = { SERVER.address },
}

load_app {
    path = make_path('audio'),
    args = { SERVER.address },
}

load_app {
    path = make_path('spotify'),
    args = { SERVER.address },
}
