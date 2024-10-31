local function volume()
    return {
        data = {
            Text {
                text = AUDIO.IsMuted and 'Muted' or AUDIO.Volume,
                position = { x = 0, y = 0 },
                size = { width = SCREEN.Width, height = SCREEN.Height / 2 },
                modifiers = { font_size = 27 },
            },
            Text {
                text = AUDIO.Name,
                position = { x = 0, y = SCREEN.Height / 2 },
                size = { width = SCREEN.Width, height = SCREEN.Height / 2 - 4 },
                modifiers = { scrolling = true },
            },
        },
        duration = 2000,
        repeats = 'Once',
    }
end

-- 5s duration on Windows due to an issue mentioned in oled-applications/media/README.md
local SPOTIFY_DURATION = PLATFORM.Os == 'windows' and 5000 or 1000
local function spotify()
    return {
        data = {
            Bar {
                value = SPOTIFY.Progress * 100.0 / SPOTIFY.Duration,
                position = { x = 0, y = 0 },
                size = { width = SCREEN.Width, height = 2 },
            },
            Text {
                text = string.format("%s - %s", SPOTIFY.Artist, SPOTIFY.Title),
                position = { x = 0, y = 1 },
                size = { width = SCREEN.Width, height = 16 },
                modifiers = { scrolling = true },
            },
            Text {
                text = string.format("%02d:%02d", CLOCK.Hours, CLOCK.Minutes),
                position = { x = 0, y = SCREEN.Height - 16 },
                size = { width = 50, height = 12 },
                modifiers = { font_size = 16 },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month], CLOCK.MonthDay),
                position = { x = SCREEN.Width - 50, y = SCREEN.Height - 16 },
                size = { width = 50, height = 12 },
                modifiers = { font_size = 16 },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = { x = 0, y = SCREEN.Height - 2 },
                size = { width = SCREEN.Width, height = 2 },
            },
        },
        duration = SPOTIFY_DURATION,
        repeats = 'ToFit',
    }
end

local function clock()
    return {
        data = {
            Text {
                text = string.format("%02d", CLOCK.Hours),
                position = { x = 10, y = 1 },
                size = { width = 54, height = 35 },
                modifiers = { font_size = 47 },
            },
            Text {
                text = string.format("%02d", CLOCK.Minutes),
                position = { x = 64, y = 0 },
                size = { width = 54, height = 26 },
                modifiers = { font_size = 36 },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month], CLOCK.MonthDay),
                position = { x = 66, y = 27 },
                size = { width = 54, height = 10 },
                modifiers = { font_size = 14 },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
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
    if CLOCK.Seconds % 20 < 10 then
        value = string.format("% 3d", math.round(WEATHER.Temperature))
        unit = '°C'
    else
        value = string.format("% 3d", math.round(WEATHER.WindSpeed))
        unit = 'km/h'
    end

    return {
        data = {
            Image {
                image = WEATHER[WEATHER.ImageKey],
                position = { x = 0, y = 0 },
                size = { width = SCREEN.Height, height = SCREEN.Height },
            },
            Text {
                text = value,
                position = { x = SCREEN.Height + 4, y = 0 },
                size = { width = SCREEN.Height * 2, height = 25 },
                modifiers = { font_size = 30 },
            },
            Text {
                text = unit,
                position = { x = 98, y = 0 },
                size = { width = 30, height = 11 },
            },
            Text {
                text = string.format("%.3s %02d:%02d", CLOCK.DayNames[CLOCK.WeekDay], CLOCK.Hours, CLOCK.Minutes),
                position = { x = SCREEN.Height + 4, y = SCREEN.Height / 2 - 2 },
                size = { width = SCREEN.Height * 2, height = SCREEN.Height / 2 - 2 },
                modifiers = { font_size = 14 },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = { x = 0, y = SCREEN.Height - 2 },
                size = { width = SCREEN.Width, height = 2 },
            },
        },
        duration = 1000,
    }
end

SCREEN_BUILDER
        :new('Steelseries Apex 7 TKL')
        :with_screen({
    {
        layout = volume,
        run_on = { 'AUDIO.IsMuted', 'AUDIO.Name', 'AUDIO.Volume' },
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
        :with_screen({
    {
        layout = weather,
        run_on = { 'CLOCK.Seconds' },
    }
})
        :with_screen_toggle({ 'KEY(RAlt)', 'KEY(Slash)' })
        :register()
