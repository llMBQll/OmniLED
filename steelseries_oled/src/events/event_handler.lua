EVENT_HANDLER = {}

EVENT_HANDLER.user_scripts = {}
EVENT_HANDLER.to_update = {}
EVENT_HANDLER.last_priority = 0
EVENT_HANDLER.last_script = {}
EVENT_HANDLER.repeat_to_fit = false
EVENT_HANDLER.repeat_once = false
EVENT_HANDLER.time_remaining = 0
EVENT_HANDLER.DEFAULT_UPDATE_TIME = 1000
EVENT_HANDLER.listeners = {}

EVENTS:add_listener(
    function(event, _data)
        local listeners = EVENT_HANDLER.listeners[event]
        if listeners ~= nil then
            for _, listener in ipairs(listeners) do
                listener()
            end
        end
    end
)

-- TODO pass data from incoming requests directly to lua
-- Swap the data vectors on update to minimize downtime on update

function EVENT_HANDLER:register_user_script(script, sensitivity_list, screens)
    if self.user_scripts[script] ~= nil then
        LOG.warn('Script ' .. script .. ' was already registered! Skipping...')
        return
    end
    self.user_scripts[script] = true
    table.insert(self.to_update, false)

    local priority = #self.to_update
    local renderer = RENDERER_FACTORY:create()

    local function slot()
        local function wrapper()
            for _, screen in ipairs(screens) do
                local env = SCRIPT_HANDLER.env
                local size = SCREENS:size(screen)
                env["SCREEN"] = size

                local result = script()
                self.last_priority = priority

                -- TODO verify result
                if result then
                    self.time_remaining = result.duration or self.DEFAULT_DURATION
                    local end_auto_repeat, image = renderer:render(priority, size, result.data)
                    SCREENS:update(screen, image)

                    local repeat_to_fit = result.repeat_to_fit or false
                    local repeat_once = result.repeat_once or false

                    if repeat_to_fit then
                        self.last_script = wrapper
                        self.repeat_to_fit = true
                    elseif repeat_once and not end_auto_repeat then
                        self.last_script = wrapper
                        self.repeat_once = true
                    else
                        self.last_script = {}
                        self.repeat_to_fit = false
                        self.repeat_once = false
                    end
                end
            end
        end

        self.to_update[priority] = wrapper
    end

    for _, key in ipairs(sensitivity_list) do
        if self.listeners[key] == nil then
            self.listeners[key] = {}
        end

        table.insert(self.listeners[key], slot)
    end
end

function EVENT_HANDLER:update(time_passed)
    if self.time_remaining > time_passed then
        self.time_remaining = self.time_remaining - time_passed
    else
        self.time_remaining = 0
    end

    for priority, script in ipairs(self.to_update) do
        if self.time_remaining > 0 and self.last_priority < priority then
            if self.repeat_to_fit then
                -- execute last script but don't bump the remaining time, this we keep proper priority and ensure
                -- the event actually expires after specified time, not repeating indefinitely
                -- (would block lower priority otherwise)
                local time = self.time_remaining
                self.last_script()
                self.time_remaining = time
            end
            break
        end

        if self.last_priority == priority and self.repeat_once then
            self.last_script()
            break
        end

        if script ~= false then
            script()
            break
        end
    end

    for priority, _ in ipairs(self.to_update) do
        self.to_update[priority] = false
    end
end

function EVENT_HANDLER:send_value(application_name, variable_name, value)
    local env = SCRIPT_HANDLER.env

    if env[application_name] == nil then
        env[application_name] = {}
    end

    -- TODO consider running updates only if the value has changed
    -- local old_value = env[application_name][variable_name]
    -- if value == old_value then
    --     return
    -- end
    env[application_name][variable_name] = value

    local key = application_name .. '.' .. variable_name
    EVENTS(key, value)
end

function EVENT_HANDLER:reset()
    self.last_priority = 0
    self.last_script = {}
    self.repeat_to_fit = false
    self.repeat_once = false
    self.time_remaining = 0
end