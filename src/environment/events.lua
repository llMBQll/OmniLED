EVENTS = {}

Event = {}

function Event:new(name)
    obj = obj or {}
    setmetatable(obj, Event)
    self.__index = self

    if obj.name == nil then
        obj.name = name
    end
    obj.slots = {}

    return obj
end

function Event:connect(slot)
    self.slots[slot] = true
end

function Event:disconnect(slot)
    self.slots[slot] = nil
end

function Event:emit(data)
    for slot, _ in pairs(self.slots) do
        slot(data)
    end
end