local function volume()
    return {
        data = {
            Text {
                text = AUDIO.IsMuted and 'Muted' or AUDIO.Volume,
                position = {
                    origin = { x = 0, y = 0 },
                    size = { width = SCREEN.Width, height = SCREEN.Height / 2 },
                },
                modifiers = { font_size = 27 },
            },
            Text {
                text = AUDIO.Name,
                position = {
                    origin = { x = 0, y = SCREEN.Height / 2 },
                    size = { width = SCREEN.Width, height = SCREEN.Height / 2 - 4 },
                },
                modifiers = { scrolling = true },
            },
        },
        duration = 2000,
        repeats = 'Once',
    }
end

local SPOTIFY_DURATION = PLATFORM.Os == 'windows' and 5000 or 1000
local function spotify()
    return {
        data = {
            Bar {
                value = SPOTIFY.Progress * 100.0 / SPOTIFY.Duration,
                position = {
                    origin = { x = 0, y = 0 },
                    size = { width = SCREEN.Width, height = 2 },
                },
            },
            Text {
                text = string.format("%s - %s", SPOTIFY.Artist, SPOTIFY.Title),
                position = {
                    origin = { x = 0, y = 1 },
                    size = { width = SCREEN.Width, height = 16 },
                },
                modifiers = { scrolling = true },
            },
            Text {
                text = string.format("%02d:%02d", CLOCK.Hours, CLOCK.Minutes),
                position = {
                    origin = { x = 0, y = SCREEN.Height - 16 },
                    size = { width = 50, height = 12 },
                },
                modifiers = { font_size = 16 },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month], CLOCK.MonthDay),
                position = {
                    origin = { x = SCREEN.Width - 50, y = SCREEN.Height - 16 },
                    size = { width = 50, height = 12 },
                },
                modifiers = { font_size = 16 },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = {
                    origin = { x = 0, y = SCREEN.Height - 2 },
                    size = { width = SCREEN.Width, height = 2 },
                },
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
                position = {
                    origin = { x = 10, y = 1 },
                    size = { width = 54, height = 35 },
                },
                modifiers = { font_size = 47 },
            },
            Text {
                text = string.format("%02d", CLOCK.Minutes),
                position = {
                    origin = { x = 64, y = 0 },
                    size = { width = 54, height = 26 },
                },
                modifiers = { font_size = 36 },
            },
            Text {
                text = string.format("%.3s%02d", CLOCK.MonthNames[CLOCK.Month], CLOCK.MonthDay),
                position = {
                    origin = { x = 66, y = 27 },
                    size = { width = 54, height = 10 },
                },
                modifiers = { font_size = 14 },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = {
                    origin = { x = 0, y = SCREEN.Height - 2 },
                    size = { width = SCREEN.Width, height = 2 },
                },
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
                position = {
                    origin = { x = 0, y = 0 },
                    size = { width = SCREEN.Height, height = SCREEN.Height },
                },
            },
            Text {
                text = value,
                position = {
                    origin = { x = SCREEN.Height + 4, y = 0 },
                    size = { width = SCREEN.Height * 2, height = 25 },
                },
                modifiers = { font_size = 30 },
            },
            Text {
                text = unit,
                position = {
                    origin = { x = 98, y = 0 },
                    size = { width = 30, height = 11 },
                },
            },
            Text {
                text = string.format("%.3s %02d:%02d", CLOCK.DayNames[CLOCK.WeekDay], CLOCK.Hours, CLOCK.Minutes),
                position = {
                    origin = { x = SCREEN.Height + 4, y = SCREEN.Height / 2 - 2 },
                    size = { width = SCREEN.Height * 2, height = SCREEN.Height / 2 - 2 },
                },
                modifiers = { font_size = 14 },
            },
            Bar {
                value = CLOCK.Seconds * 100.0 / 59,
                position = {
                    origin = { x = 0, y = SCREEN.Height - 2 },
                    size = { width = SCREEN.Width, height = 2 },
                },
            },
        },
        duration = 1000,
    }
end

-- local current_screen = 1
-- local max_screens = 2
-- SHORTCUTS:register(
--     { 'KEY(RAlt)', 'KEY(Slash)' },
--     function()
--         if current_screen == max_screens then
--             current_screen = 1
--         else
--             current_screen = current_screen + 1
--         end
--         LOG:debug('Current screen: ' .. current_screen)
--     end,
--     SHORTCUTS.RESET_STATE
-- )

-- function current_screen_is(screen)
--     return function()
--         return current_screen == screen
--     end
-- end

-- local screen_predicate = current_screen_is(1)
-- local count_predicate = PREDICATE.Times(1)

-- local volume_script = {
--     action = volume,
--     predicate = function()
--         return screen_predicate() and count_predicate()
--     end,
--     run_on = { 'AUDIO.IsMuted', 'AUDIO.Name', 'AUDIO.Volume' },
-- }

-- local spotify_script = {
--     action = spotify,
--     predicate = current_screen_is(1),
--     run_on = { 'SPOTIFY.Artist', 'SPOTIFY.Progress', 'SPOTIFY.Title' },
-- }

-- local clock_script = {
--     action = clock,
--     predicate = current_screen_is(1),
--     run_on = { 'CLOCK.Seconds' },
-- }

-- local weather_script = {
--     action = weather,
--     predicate = current_screen_is(2),
--     run_on = { 'CLOCK.Seconds' },
-- }

-- register('Steelseries Apex 7 TKL', { volume_script, spotify_script, clock_script, weather_script })


local DEVICE_BUILDER = {}

function DEVICE_BUILDER:new(name)
    local new_object = {}
    self.__index = self
    setmetatable(new_object, self)
    new_object.name = name
    new_object.scripts = {}
    new_object.shortcut = {}
    new_object.screen_count = 0
    new_object.current_screen = 1
    return new_object
end

function DEVICE_BUILDER:with_screen(screen)
    -- TODO assert no 'with_script' calls

    self.screen_count = self.screen_count + 1
    local screen_number = self.screen_count
    for _, script in ipairs(screen) do
        -- Wrap predicate within screen predicate
        local predicate = script.predicate
        local wrapped_predicate = function()
            if self.current_screen == screen_number and (predicate == nil or predicate()) then
                return true
            end
            return false
        end
        script.predicate = wrapped_predicate

        table.insert(self.scripts, script)
    end

    return self
end

function DEVICE_BUILDER:with_script(script)
    -- TODO assert no 'with_screen' calls

    table.insert(self.scripts, script)

    return self
end

function DEVICE_BUILDER:with_screen_toggle(shortcut)
    self.shortcut = shortcut

    return self
end

function DEVICE_BUILDER:enable()
    SHORTCUTS:register(
            self.shortcut,
            function()
                reset(self.name)
                if self.current_screen == self.screen_count then
                    self.current_screen = 1
                else
                    self.current_screen = self.current_screen + 1
                end
                LOG:debug('Current screen: ' .. self.current_screen)
            end
    )

    register(self.name, self.scripts)
end

local DEVICES = {}

function DEVICES:find(name)
    -- TODO actually find the device instead of pretending
    return DEVICE_BUILDER:new(name)
end

DEVICES:find('Steelseries Apex 7 TKL')
       :with_screen({
    {
        action = volume,
        predicate = PREDICATE.Times(1),
        run_on = { 'AUDIO.IsMuted', 'AUDIO.Name', 'AUDIO.Volume' },
    },
    {
        action = spotify,
        run_on = { 'SPOTIFY.Artist', 'SPOTIFY.Progress', 'SPOTIFY.Title' },
    },
    {
        action = clock,
        run_on = { 'CLOCK.Seconds' },
    },
})
       :with_screen({
    {
        action = weather,
        run_on = { 'CLOCK.Seconds' },
    }
})
       :with_screen_toggle({ 'KEY(RAlt)', 'KEY(Slash)' })
       :enable()

--SHORTCUTS:register(
--        { 'KEY(RAlt)', 'KEY(Slash)' },
--        function()
--            LOG:debug('Omegalul')
--        end
--)