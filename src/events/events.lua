EVENTS = {}

Event = {}

function Event:register(name)
    LOG.trace('Registering "' .. name .. '" event')

    if EVENTS[name] ~= nil then
        return EVENTS[name]
    end

    obj = {}
    setmetatable(obj, Event)
    self.__index = self

    if obj.name == nil then
        obj.name = name
    end
    obj.slots = {}

    EVENTS[name] = obj

    return obj
end

function Event:connect(slot)
    LOG.trace('Connecting ' .. tostring(slot) .. ' to "' .. self.name .. '"')

    self.slots[slot] = true
end

function Event:disconnect(slot)
    LOG.trace('Disconnecting ' .. tostring(slot) .. ' from "' .. self.name .. '"')

    self.slots[slot] = nil
end

function Event:emit(...)
    local data = '(' .. table.concat({...}, ', ') .. ')'
    LOG.trace('Emitting "' .. self.name .. '" with ' .. data)

    for slot, _ in pairs(self.slots) do
        slot(...)
    end
end