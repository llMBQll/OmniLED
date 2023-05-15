local function register(fn, sensitivity_list)
    UPDATE_HANDLER:register_user_script(fn, sensitivity_list)
end

local function get_sandbox_env()
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
        register = register,
        print = print,
    }

    return sandbox_env
end

sandbox = get_sandbox_env()

-- Compile a handler and assign it to a context with a given name
function compile(chunk)
    local fn, err = loadfile('scripts.lua', 't', sandbox)

    if err then
        print(err)
        return err
    end

    fn()
end