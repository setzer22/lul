local color_table = {
    black = "\27[30m",
    red = "\27[31m",
    green = "\27[32m",
    yellow = "\27[33m",
    blue = "\27[34m",
    magenta = "\27[35m",
    cyan = "\27[36m",
    white = "\27[37m",
    reset = "\27[0m",
}

function colored(color, text)
    return color_table[color] .. text .. color_table.reset
end

function log_command(cmd, opts)
    if opts and opts.silent then 
        return
    else
        print(colored("yellow", "-- " .. cmd))
    end
end

function abort_if_failed(status, opts)
    if status ~= 0 and not opts.allowfail then
        print(colored("red", "Command failed, aborting. If this was unintended, pass the 'allowfail' option"))
        os_exit(status)
    end
end


function sh(cmd, opts)
    opts = opts or {}

    log_command(cmd, opts)
    local result = os_shell(cmd, false)
    abort_if_failed(result.status, opts)
    
    return result;
end

function sh_out(cmd, opts)
    opts = opts or {}

    log_command(cmd, opts)
    
    local result = os_shell(cmd, true)
    
    abort_if_failed(result.status, opts)

    return result
end

function sh_all(commands, opts)
    opts = opts or {}

    local results = {}
    for _, cmd in ipairs(commands) do
        local result = sh(cmd, opts);
        table.insert(results, result)
    end
    
    return results
end

function with_dir(path, fn)
    local current = workdir()
    cd(path)
    fn()
    cd(current)
end

function combine_paths(...)
    local result = ""
    local first = true
    for _, path in ipairs({...}) do
        if first or path:sub(1, 1) == '/' then
            result = path
        else
            if result:sub(-1) == '/' then
                result = result .. path
            else
                result = result .. '/' .. path
            end
        end
        first = false
    end
    return result
end

getmetatable("").__div = combine_paths

HOME = getenv('HOME')