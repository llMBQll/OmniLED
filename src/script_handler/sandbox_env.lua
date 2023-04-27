sandbox = get_sandbox_env()
contexts = {}
updated = {}
last_priority = 0
time_remaining = 0

function get_sandbox_env()
    local sandbox_env = {
        ipairs = ipairs,
        next = next,
        pairs = pairs,
        pcall = pcall,
        tonumber = tonumber,
        tostring = tostring,
        type = type,
        unpack = unpack,
        coroutine = { create = coroutine.create, resume = coroutine.resume, running = coroutine.running, status = coroutine.status, wrap = coroutine.wrap },
        string = { byte = string.byte, char = string.char, find = string.find, format = string.format, gmatch = string.gmatch, gsub = string.gsub, len = string.len, lower = string.lower, match = string.match, rep = string.rep, reverse = string.reverse, sub = string.sub, upper = string.upper },
        table = { insert = table.insert, maxn = table.maxn, remove = table.remove, sort = table.sort },
        math = { abs = math.abs, acos = math.acos, asin = math.asin, atan = math.atan, atan2 = math.atan2, ceil = math.ceil, cos = math.cos, cosh = math.cosh, deg = math.deg, exp = math.exp, floor = math.floor, fmod = math.fmod, frexp = math.frexp, huge = math.huge, ldexp = math.ldexp, log = math.log, log10 = math.log10, max = math.max, min = math.min, modf = math.modf, pi = math.pi, pow = math.pow, rad = math.rad, random = math.random, sin = math.sin, sinh = math.sinh, sqrt = math.sqrt, tan = math.tan, tanh = math.tanh },
        os = { clock = os.clock, difftime = os.difftime, time = os.time },
    }

    return sandbox_env
end

function get_default_context()
    local ctx = {
        handlers = {},
        sensitivity_list = {},
        scrolling_text = {},
        reset_count = false,
    }

    return ctx
end

-- Compile a handler and assign it to a context with a given name
function compile(name, chunk)
    local fn, err = load(chunk, nil, 't', sandbox)

    if err then
        return err
    end

    local ctx = contexts[name]
    table.insert(ctx.handlers, fn)
    table.insert(ctx.scrolling_text, { text = '', count = 0 })
    return nil
end

-- Set the value by table_name and value_name so it can be found as table_name.value_name later
function set_value(table_name, value_name, value)
    if sandbox[table_name] == nil then
        sandbox[table_name] = {}
    end
    sandbox[table_name][value_name] = value

    updated[table_name .. '.' .. value_name] = true
end

function Text(text, format)
    return { 'text', tostring(text), format }
end

function ScrollingText(text, format)
    local scrolling_text = ctx['scrolling_text'][ctx.id]

    if text == previous_text then
        scrolling_text.count = scrolling_text.count + 1
    else
        scrolling_text.text = text
        scrolling_text.count = 0
        ctx.reset_count = true
    end

    return { 'scrolling-text', text, scrolling_text.count, format }
end

function Bar(percentage, format)
    if type(percentage) ~= 'number' then
        return { 'error', 'Given argument is not a number' }
    end
    return { 'bar', percentage, format }
end

-- Set the context associated with the given name and execute all the handlers
function call(ctx)
    results = {}

    sandbox['ctx'] = ctx

    for i, fn in ipairs(ctx.handlers) do
        ctx.id = i
        table.insert(results, fn())
    end

    if ctx.reset_count == true then
        for _, scrolling_text in ipairs(ctx.scrolling_text) do
            scrolling_text.count = 0
        end
        ctx.reset_count = false
    end

    return results
end

-- Process all updates and fire appropriate handlers, specify the interval between calls
function process_updates(interval)
    local priority = 0
    time_remaining = time_remaining > interval and time_remaining - interval or 0

    for _, ctx in ipairs(contexts) do
        if last_priority < priority and time_remaining ~= 0 then
            return {}
        end
        for _, value in ipairs(updated) do
            if contexts.sensitivity_list[value] ~= nil then
                last_priority = priority
                time_remaining = 2000
                updated = {} -- by now every update is 'outdated' so the array is discarded
                return call(ctx)
            end
        end
    end
    return {}
end

function format_display(fragments, display)

end

-- Clear context data for all handlers
function clear()
    contexts = {}
    updated = {}
    last_priority = 0
    time_remaining = 0
end