pub const LUA_CODE: &[u8] = r#"
-- Polyfills

local print
local require

local do_init
do_init = function()
    local inventory = {}
    local function swing(side)
        local success, block = piccolo_component_invoke('$robot', 'robot.swing', side)
        if success then
            inventory[block] = (inventory[block] or 0) + 1
            
            local name = piccolo_component_invoke('$robot', 'robot.name')
            print(name .. "'s inventory")
            for k, count in pairs(inventory) do
                print('- ' .. k .. ': ' .. tostring(count))
            end
            print('')
        end
        return success
    end
    
    
    local sides = {
        down = 0,
        up = 1,
        back = 2,
        front = 3
    }
    local robot = {
        inventorySize = function() return math.huge end,
        setLightColor = function(c) return
            piccolo_component_invoke('$gpu', 'gpu.setForeground', c) end,
        forward = function() return
            piccolo_component_invoke('$robot', 'robot.move', sides.front) end,
        back = function() return
            piccolo_component_invoke('$robot', 'robot.move', sides.back) end,
        up = function() return
            piccolo_component_invoke('$robot', 'robot.move', sides.up) end,
        down = function() return
            piccolo_component_invoke('$robot', 'robot.move', sides.down) end,
        turnRight = function() return
            piccolo_component_invoke('$robot', 'robot.turn', true) end,
        turnAround = function() return
            piccolo_component_invoke('$robot', 'robot.hidden$turnaround') end,
        turnLeft = function() return
            piccolo_component_invoke('$robot', 'robot.turn', false) end,
        select = function() end,
        count = function() return math.huge end,
        swing = function() return
            piccolo_component_invoke('$robot', 'robot.swing', sides.front) end,
        swingUp = function() return
            piccolo_component_invoke('$robot', 'robot.swing', sides.up) end,
        swingDown = function() return
            piccolo_component_invoke('$robot', 'robot.swing', sides.down) end,
        place = function() return false end,
        space = function() return math.huge end,
        drop = function() return false end,
        dropDown = function() return false end,
        detectUp = function() return
            not piccolo_component_invoke('$robot', 'robot.hidden$compareAir', sides.up), 'solid' end,
        detect = function() return
            not piccolo_component_invoke('$robot', 'robot.hidden$compareAir', sides.front), 'solid' end,
        detectDown = function() return
            not piccolo_component_invoke('$robot', 'robot.hidden$compareAir', sides.down), 'solid' end,
        compareUp = function() return
            piccolo_component_invoke('$robot', 'robot.hidden$compareRocks', sides.up) end,
        compare = function() return
            piccolo_component_invoke('$robot', 'robot.hidden$compareRocks', sides.front) end,
        compareDown = function() return
            piccolo_component_invoke('$robot', 'robot.hidden$compareRocks', sides.down) end
    }
    robot.swing = function() return swing(sides.front) end
    robot.swingUp = function() return swing(sides.up) end
    robot.swingDown = function() return swing(sides.down) end
    
    local component = {
        isAvailable = function() return false end
    }
    local computer = {
        maxEnergy = function() return 4096 * 30 end,
        energy = function() return 4096 * 30 end
    }
    
    require = function(lib)
        if lib == 'event' then return nil end
        if lib == 'robot' then return robot end
        if lib == 'component' then return component end
        if lib == 'computer' then return computer end
        if lib == 'sides' then return sides end
    end
    
    local print_y = 0
    local _w, print_h = piccolo_component_invoke('$gpu', 'gpu.getResolution')
    local s8 = '        '
    local s16 = s8 .. s8
    local s32 = s16 .. s16
    local s80 = s16 .. s32 .. s32
    print = function(s)
        print_y = print_y + 1
        if print_y > print_h then print_y = 1 end
        piccolo_component_invoke('$gpu', 'gpu.set', 1, print_y, tostring(s) .. s80, false)
    end
    
    tonumber = tonumber or function() return nil end
    
    string.format = function(...)
        local a = {...}
        local b = {}
        for k, v in pairs(a) do b[k] = tostring(v) end
        return table.concat(b, ' \t')
    end
    
    robot.turnLeft()
    
    do_init = nil
end
do_init()




local r = require('robot')

-- Horizontal orientation:
-- 0 = front, 1 = right, 2 = back, 3 = left
local facing = 0

local x = 0
local y = 0
local z = 0

local visited = {}
local oreMined = 0
local trashMined = 0

local dirs = {
    { dx = 0, dz = -1, kind = 0 },
    { dx = 1, dz = 0, kind = 1 },
    { dx = 0, dz = 1, kind = 2 },
    { dx = -1, dz = 0, kind = 3 }
}

local function key(px, py, pz)
    return px .. ":" .. py .. ":" .. pz
end

local function turnTo(direction)
    local delta = (direction - facing) % 4

    if delta == 1 then
        r.turnRight()
    elseif delta == 2 then
        r.turnAround()
    elseif delta == 3 then
        r.turnLeft()
    end

    facing = direction
end

local function moveHorizontal(direction)
    turnTo(direction)

    local ok = r.forward()

    if ok then
        if direction == 0 then
            z = z - 1
        elseif direction == 1 then
            x = x + 1
        elseif direction == 2 then
            z = z + 1
        else
            x = x - 1
        end
    end

    return ok
end

local function moveVertical(amount)
    local ok

    if amount > 0 then
        ok = r.up()
    else
        ok = r.down()
    end

    if ok then
        y = y + amount
    end

    return ok
end

local function horizontalDetect(direction)
    turnTo(direction)
    return r.detect()
end

local function horizontalCompare(direction)
    turnTo(direction)
    return r.compare()
end

local function horizontalSwing(direction)
    turnTo(direction)
    return r.swing()
end

local function mineVerticalRun()
    -- Mine the complete ore run upward, then return.
    local moved = 0

    while r.detectUp() and not r.compareUp() do
        local dug = r.swingUp()

        if not dug then
            break
        end

        if not moveVertical(1) then
            break
        end

        moved = moved + 1
        oreMined = oreMined + 1
    end

    while moved > 0 do
        moveVertical(-1)
        moved = moved - 1
    end

    -- Mine the complete ore run downward, then return.
    moved = 0

    while r.detectDown() and not r.compareDown() do
        local dug = r.swingDown()

        if not dug then
            break
        end

        if not moveVertical(-1) then
            break
        end

        moved = moved + 1
        oreMined = oreMined + 1
    end

    while moved > 0 do
        moveVertical(1)
        moved = moved - 1
    end
end

local function mineOre(direction)
    if direction == 4 then
        if not r.detectUp() or r.compareUp() then
            return false
        end

        if not r.swingUp() then
            return false
        end

        if not moveVertical(1) then
            return false
        end

        oreMined = oreMined + 1
        mineVerticalRun()
        moveVertical(-1)
        return true
    end

    if direction == 5 then
        if not r.detectDown() or r.compareDown() then
            return false
        end

        if not r.swingDown() then
            return false
        end

        if not moveVertical(-1) then
            return false
        end

        oreMined = oreMined + 1
        mineVerticalRun()
        moveVertical(1)
        return true
    end

    if not horizontalDetect(direction) then
        return false
    end

    if horizontalCompare(direction) then
        return false
    end

    if not horizontalSwing(direction) then
        return false
    end

    if not moveHorizontal(direction) then
        return false
    end

    oreMined = oreMined + 1
    mineVerticalRun()
    moveHorizontal((direction + 2) % 4)

    return true
end

local function enterTrash(direction)
    if not horizontalDetect(direction) then
        return false
    end

    if not horizontalCompare(direction) then
        return false
    end

    -- A failed swing means that the block is an unbreakable boundary,
    -- such as bedrock. Do not enter it.
    if not horizontalSwing(direction) then
        return false
    end

    if not moveHorizontal(direction) then
        return false
    end

    trashMined = trashMined + 1
    return true
end

local function explore()
    local here = key(x, y, z)

    if visited[here] then
        return
    end

    visited[here] = true

    -- First mine every immediately visible ore block.
    -- Vertical continuation is mined automatically.
    local d

    for d = 0, 3 do
        mineOre(d)
    end

    mineOre(4)
    mineOre(5)

    -- Explore already open horizontal cells before digging trash.
    for d = 0, 3 do
        if not horizontalDetect(d) then
            local oldX = x
            local oldY = y
            local oldZ = z

            if moveHorizontal(d) then
                explore()
                moveHorizontal((d + 2) % 4)

                x = oldX
                y = oldY
                z = oldZ
            end
        end
    end

    -- Only open trash horizontally. This avoids wasting blocks above or
    -- below a column; once ore is found, its entire vertical run is mined.
    for d = 0, 3 do
        local oldX = x
        local oldY = y
        local oldZ = z

        if enterTrash(d) then
            explore()
            moveHorizontal((d + 2) % 4)

            x = oldX
            y = oldY
            z = oldZ
        end
    end
end

explore()



print('DONE')
while true do piccolo_yield() end
"#.as_bytes();

