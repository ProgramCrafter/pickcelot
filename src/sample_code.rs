/*
const _LUA_CODE_SPECTRUM: &[u8] = "
string.rep = function(s, n)
    if n == 0 then return '' end
    if n == 1 then return s end
    local t = ((n % 2) == 1) and s or ''
    local u = string.rep(s, math.floor(n / 2))
    return (t .. u) .. u
end

local gpu = {
    set = function(x, y, s, d)
        if d == nil then d = false end
        return piccolo_component_invoke('$gpu', 'gpu.set', x, y, s, d)
    end,
    getResolution = function()
        return piccolo_component_invoke('$gpu', 'gpu.getResolution')
    end,
    setBackground = function(c)
        return piccolo_component_invoke('$gpu', 'gpu.setBackground', c)
    end,
    setForeground = function(c)
        return piccolo_component_invoke('$gpu', 'gpu.setForeground', c)
    end
}
local event = {
    listen = function() end
}
local term = {
    clear = function()
        gpu.setBackground(0)
        local w, h = gpu.getResolution()
        local s = string.rep(' ', w)
        for i = 1, h, 1 do
            gpu.set(1, i, s, false)
        end
        gpu.setForeground(0xFFFFFF)
    end
}
local ost = 0.0
local os = {
    sleep = function(t)
        if t <= 0.0 then return piccolo_yield() end
        ost = ost + t
        return piccolo_component_invoke('$os', 'os.sleep', t)
    end,
    time = function() ost = ost + 1.0 return ost end
}
local hologram
hologram = {
    palette = {[0]={0,0,0},[3]={0,40,255}},
    buffer = {},
    history = {},
    setPaletteColor = function(i, c)
        hologram.palette[i] = {(c-(c%65536))/65536,
                               ((c-(c%256))/256)%256, c%256}
    end,
    set = function(x, y, z, i)
        local key = x*100+y
        local power = 1/3 / (2^math.abs(z-11))
        local c = hologram.palette[i] or {0xFF,0,0xFF}
        local d = hologram.history[key*100+z] or {0,0,0}
        if c == d then return end
        hologram.history[key*100+z] = c
        
        if hologram.buffer[key]==nil then hologram.buffer[key]={0,0,0} end
        local v = hologram.buffer[key]
        
        v[1] = v[1] + (c[1] - d[1]) * power
        v[2] = v[2] + (c[2] - d[2]) * power
        v[3] = v[3] + (c[3] - d[3]) * power
        
        local p,q,r = math.floor(v[1]),math.floor(v[2]),math.floor(v[3])
        if p < 0 then p = 0 end
        if q < 0 then q = 0 end
        if r < 0 then r = 0 end
        if p > 255 then p = 255 end
        if q > 255 then q = 255 end
        if r > 255 then r = 255 end
        gpu.setBackground(p*65536+q*256+r)
        gpu.set(x, y, ' ')
    end,
    clear = term.clear
}
local computer = {}


-- GPU Waveform Demo
-- Tests gpu.set, gpu.setBackground, gpu.setForeground with different parameters
-- Y axis runs down, screen size 90x35 characters

-- Pre-build fixed strings to avoid string.rep (only string.sub allowed)
local SPACE35 = '                                   '  -- 35 spaces
local PIPE35  = '|||||||||||||||||||||||||||||||||||'  -- 35 pipes

local phase = 0
local center = 21.5
local amplitude = 11
local frequency = 3  -- waves across the width

local w, h = gpu.getResolution()

gpu.setBackground(0)

-- Main animation loop
while true do
    -- Clear the whole screen column by column using vertical space strings
    gpu.setForeground(0xFFFFFF)
    --[[
    for y = 1, h do
        gpu.set(1, y, string.rep(' ', w))  -- vertical placement, clears column
    end
    --]]

    -- Draw title (horizontal placement)
    gpu.set(25, 1, 'GPU Waveform Demo', true)
    gpu.set(15, 3, 'place_vertically=true for bars, nil here')

    -- Draw waveform bars with vertical text + point markers
    for x = 1, w do
        local wave = center + amplitude * math.sin((x / w) * 2 * math.pi * frequency + phase)
        local y_wave = math.floor(wave + 0.5)
        if y_wave < 1 then y_wave = 1
        elseif y_wave > h then y_wave = h end

        -- Dynamic colour based on bar height (blue -> red)
        local r = math.floor(255 * (y_wave / h) ^ 0.5)
        local g = math.floor(255 * math.max(0, y_wave - 29) / 12)
        local b = math.floor(255 * (1 - y_wave / h) ^ 0.5)
        local color = r * 65536 + g * 256 + b

        gpu.setForeground(color)

        -- Vertical bar using place_vertically = true
        -- Height = y_wave, starts at row 1, goes downwards
        local bar = string.sub(PIPE35, 1, y_wave)
        local ter = string.sub(SPACE35, 1, amplitude + center + 0.5 - y_wave)
        gpu.set(x, 1, bar .. ter, true)

        -- Overlay the actual wave point as a solid block (horizontal placement)
        gpu.setForeground(0x24 * 0x010101)
        gpu.set(x, y_wave, '█', false)
    end

    -- Add a moving horizontal label at the bottom to test horizontal placement again
    gpu.setForeground(0xFFFF00)
    local label = ' Phase: ' .. tostring(phase) .. '   '
    local labelX = math.floor(45 + 20 * math.sin(phase * 0.7))
    gpu.set(labelX, 34, label, false)

    os.sleep(0.07)
    phase = phase + 0.0625
end
".as_bytes();
*/

pub const LUA_CODE: &[u8] = b"

local me = piccolo_component_invoke('$robot', 'robot.name')
piccolo_component_invoke('$gpu', 'gpu.set', 1, 1, me .. ' operations     ', false)
piccolo_component_invoke('$gpu', 'gpu.set', 1, 2, 'Inventory:               ', false)

local inventory = {}

local function swing()
    local success, block = piccolo_component_invoke('$robot', 'robot.swing', 3)
    if success then
        inventory[block] = (inventory[block] or 0) + 1
    end
    return success
end

local moves = 0

while true do
    while swing() do
        if piccolo_component_invoke('$robot', 'robot.move', 3) then
            moves = moves + 1
        end
    end
    piccolo_component_invoke('$robot', 'robot.turn', true)
    
    local h = 2
    for itemid, count in pairs(inventory) do
        h = h + 1
        piccolo_component_invoke(
            '$gpu', 
            'gpu.set',
            1,
            h,
            '- ' .. itemid .. ': ' .. tostring(count) .. '    ',
            false
        )
    end
    piccolo_component_invoke('$gpu', 'gpu.set', 1, h + 1,
        'Moves: ' .. tostring(moves) .. '  ', false)
end
";

