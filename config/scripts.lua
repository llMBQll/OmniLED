local function volume()
    local function display_device(widgets, offset, device, device_type)
        if device then
            table.insert(widgets, Text {
                text = device.Name,
                scrolling = true,
                repeats = 'Once',
                position = { x = 0, y = offset },
                size = { width = SCREEN.Width * 2 / 3, height = SCREEN.Height / 2 },
                animation_group = 1,
            })
            table.insert(widgets, Text {
                text = device.IsMuted and ' M ' or string.format("%3d", device.Volume),
                font_size = 24,
                text_offset = 1,
                position = { x = SCREEN.Width * 2 / 3, y = offset },
                size = { width = SCREEN.Width / 3, height = SCREEN.Height / 2 },
            })
        else
            table.insert(widgets, Text {
                text = string.format('No %s device', device_type),
                scrolling = true,
                repeats = 'Once',
                position = { x = 0, y = offset },
                size = { width = SCREEN.Width, height = SCREEN.Height / 2 },
                animation_group = 1,
            })
        end
    end

    local widgets = {}
    display_device(widgets, 0, AUDIO.Output, 'output')
    display_device(widgets, SCREEN.Height / 2, AUDIO.Input, 'input')
    return {
        widgets = widgets,
        duration = 2000,
    }
end

-- 5s duration on Windows due to an issue mentioned in oled-applications/media/README.md
local SPOTIFY_DURATION = PLATFORM.Os == 'windows' and 5000 or 1000
local function spotify()
    return {
        widgets = {
            Bar {
                value = SPOTIFY.Progress,
                range = { min = 0, max = SPOTIFY.Duration },
                position = { x = 0, y = 0 },
                size = { width = SCREEN.Width, height = 2 },
            },
            Text {
                text = string.format("%s - %s", SPOTIFY.Artist, SPOTIFY.Title),
                scrolling = true,
                position = { x = 0, y = 2 },
                size = { width = SCREEN.Width, height = 20 },
            },
            Text {
                text = string.format("%02d:%02d", CLOCK.Hours, CLOCK.Minutes),
                position = { x = 0, y = SCREEN.Height - 18 },
                size = { width = 50, height = 18 },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month], CLOCK.MonthDay),
                position = { x = SCREEN.Width - 50, y = SCREEN.Height - 18 },
                size = { width = 50, height = 18 },
            },
            Bar {
                value = CLOCK.Seconds,
                range = { min = 0, max = 59 },
                position = { x = 0, y = SCREEN.Height - 2 },
                size = { width = SCREEN.Width, height = 2 },
            },
        },
        duration = SPOTIFY_DURATION,
    }
end

local function clock()
    return {
        widgets = {
            Text {
                text = string.format("%02d", CLOCK.Hours),
                font_size = 50,
                text_offset = 1,
                position = { x = 10, y = 0 },
                size = { width = SCREEN.Width / 2, height = SCREEN.Height - 3 },
            },
            Text {
                text = string.format("%02d", CLOCK.Minutes),
                font_size = 37,
                text_offset = 1,
                position = { x = SCREEN.Width / 2 + 3, y = 0 },
                size = { width = 54, height = 26 },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month], CLOCK.MonthDay),
                position = { x = SCREEN.Width / 2 + 6, y = 26 },
                size = { width = 54, height = 14 },
            },
            Bar {
                value = CLOCK.Seconds,
                range = { min = 0, max = 59 },
                position = { x = 0, y = SCREEN.Height - 2 },
                size = { width = SCREEN.Width, height = 2 },
            }
        },
        duration = 1000,
    }
end

local function weather()
    local value
    local unit
    if CLOCK.Seconds % 10 < 5 then
        value = string.format("%3d", math.round(WEATHER.Temperature))
        unit = '°' .. WEATHER.TemperatureUnit
    else
        value = string.format("%3d", math.round(WEATHER.WindSpeed))
        unit = WEATHER.WindSpeedUnit
    end

    return {
        widgets = {
            Image {
                image = WEATHER[WEATHER.ImageKey],
                position = { x = 0, y = 0 },
                size = { width = SCREEN.Height, height = SCREEN.Height },
            },
            Text {
                text = value,
                font_size = 30,
                text_offset = 1,
                position = { x = SCREEN.Height, y = 0 },
                size = { width = SCREEN.Height * 2, height = SCREEN.Height * 2 / 3 },
            },
            Text {
                text = unit,
                position = { x = 96, y = 0 },
                size = { width = 30, height = 13 },
            },
            Text {
                text = string.format("%.3s %02d:%02d", CLOCK.DayNames[CLOCK.WeekDay], CLOCK.Hours, CLOCK.Minutes),
                position = { x = SCREEN.Height + 4, y = SCREEN.Height / 2 + 4 },
                size = { width = SCREEN.Height * 2, height = SCREEN.Height / 2 - 4 },
            },
            Bar {
                value = CLOCK.Seconds,
                range = { min = 0, max = 59 },
                position = { x = 0, y = SCREEN.Height - 2 },
                size = { width = SCREEN.Width, height = 2 },
            },
        },
        duration = 1000,
    }
end

SCREEN_BUILDER
    :new('Emulator')
    :with_layout_group({
        {
            layout = volume,
            run_on = { 'AUDIO.Input', 'AUDIO.Output' },
        },
        {
            layout = spotify,
            run_on = { 'SPOTIFY.Artist', 'SPOTIFY.Progress', 'SPOTIFY.Title' },
        },
        {
            layout = clock,
            run_on = { 'CLOCK.Seconds' },
        },
    })
    :with_layout_group({
        {
            layout = weather,
            run_on = { 'CLOCK.Seconds' },
        }
    })
    :with_layout_group_toggle({ 'KEY(RAlt)', 'KEY(Slash)' })
    :register()