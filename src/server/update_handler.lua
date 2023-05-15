UPDATE_HANDLER = {}

UPDATE_HANDLER.user_scripts = {}
UPDATE_HANDLER.to_update = {}
UPDATE_HANDLER.last_priority = 0
UPDATE_HANDLER.time_remaining = 0
UPDATE_HANDLER.DEFAULT_UPDATE_TIME = 1000

-- TODO pass data from incoming requests directly to lua
-- Swap the data vectors on update to minimize downtime on update

function UPDATE_HANDLER:register_user_script(script, sensitivity_list)
    if self.user_scripts[script] ~= nil then
        LOG.warn('Function ' .. script .. ' is already registered! Skipping...')
        return
    end
    self.user_scripts[script] = true
    table.insert(self.to_update, false)

    local priority = #self.to_update

    local function slot()
        local function wrapper()
            local result = script()
            self.last_priority = priority

            -- TODO take care of this code
            --if result then
            --    self.time_remaining = result.duration or self.DEFAULT_DURATION
            --end
            --EVENTS['UPDATE_READY']:emit(result)
        end

        self.to_update[priority] = wrapper
    end

    for _, key in ipairs(sensitivity_list) do
        local event = Event:register(key)
        event:connect(slot)
    end
end

function UPDATE_HANDLER:update(time_passed)
    if self.time_remaining > time_passed then
        self.time_remaining = self.time_remaining - time_passed
    else
        self.time_remaining = 0
    end

    for priority, script in ipairs(self.to_update) do
        if self.time_remaining > 0 and self.last_priority > priority then
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

function UPDATE_HANDLER:send_value(application_name, variable_name, value)
    if _G[application_name] == nil then
        _G[application_name] = {}
    end

    -- TODO consider running updates only if the value has changed
    -- local old_value = _G[application_name][variable_name]
    _G[application_name][variable_name] = value

    local key = application_name .. '.' .. variable_name
    local event = EVENTS[key]
    if event ~= nil then
        event:emit(key)
    end
end