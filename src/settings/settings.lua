SETTINGS = {}

SETTINGS.defaults = {
    update_interval = 100,
    applications_file = 'applications.lua',
    settings_file = 'settings.lua',
    scrolling_text_ticks_at_edge = 8,
    scrolling_text_ticks_per_move = 2,
    supported_devices_file = 'devices.lua',
}

function SETTINGS:load()
    local user_input = {}
    f, err = loadfile(self.defaults['settings_file'], 't', user_input)
    if err then
        LOG.error(err)
        return
    end
    f()

    self:apply_defaults()
    self:apply_user(user_input)
end

function SETTINGS:apply_defaults()
    for key, value in pairs(self.defaults) do
        self[key] = value
    end
    LOG.trace('Default settings loaded')
end

function SETTINGS:apply_user(input)
    for key, value in pairs(input) do
        if self.defaults[key] == nil then
            LOG.warn('Unknown setting "' .. key .. "'")
        end

        LOG.debug('Set "' .. key .. '" to "' .. value .. '"')
        self[key] = value
    end
end