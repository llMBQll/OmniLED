SCRIPT_HANDLER = {}

function SCRIPT_HANDLER.make_sandbox_env()
    local function register(fn, sensitivity_list)
        if type(fn) ~= 'function' then
            LOG.warn('Expected the first argument to be "function", got "' .. type(fn) .. '"! User script not registered.')
            return
        end

        -- If we get a string - convert it to a one element array
        if type(sensitivity_list) == 'string' then
            sensitivity_list = { sensitivity_list }
        end

        -- Assert that we got a table...
        if type(sensitivity_list) ~= 'table' then
            LOG.warn('Expected the second argument to be "table", got "' .. type(sensitivity_list) .. '"! User script not registered.')
            return
        end

        -- ... and every element is a string
        for i, element in ipairs(sensitivity_list) do
            if type(element) ~= 'string' then
                LOG.warn('Expected the second argument to be an array of "string", got "' .. type(element) .. '" at index ' .. i .. '! User script not registered.')
                return
            end
        end

        UPDATE_HANDLER:register_user_script(fn, sensitivity_list)
    end

    sandbox_env = {
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

SCRIPT_HANDLER.env = SCRIPT_HANDLER.make_sandbox_env()

-- Compile a handler and assign it to a context with a given name
function SCRIPT_HANDLER:compile()
    local fn, err = loadfile('scripts.lua', 't', self.env)

    if err then
        print(err)
        return err
    end

    fn()
end