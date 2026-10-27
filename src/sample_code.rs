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



-- ==================== Mining Program ====================

local r = require('robot')

local facing = 0 -- 0 = North (-z), 1 = East (+x), 2 = South (+z), 3 = West (-x)
local x, y, z = 0, 0, 0

local function turnTo(target)
    local diff = (target - facing) % 4
    if diff == 1 then
        r.turnRight()
    elseif diff == 2 then
        r.turnAround()
    elseif diff == 3 then
        r.turnLeft()
    end
    facing = target
end

local function moveForward()
    if r.forward() then
        if facing == 0 then z = z - 1
        elseif facing == 1 then x = x + 1
        elseif facing == 2 then z = z + 1
        elseif facing == 3 then x = x - 1 end
        return true
    end
    return false
end

local function moveUp()
    if r.up() then
        y = y + 1
        return true
    end
    return false
end

local function moveDown()
    if r.down() then
        y = y - 1
        return true
    end
    return false
end

local function cellKey(cx, cy, cz)
    return cx .. ":" .. cy .. ":" .. cz
end

local function colKey(cx, cz)
    return cx .. ":" .. cz
end

local visited3D = {}
local minedCol = {}
local planeVisited = {}

local maxOreY = 0
local bestUpElevator = nil

local minOreY = 0
local bestDownElevator = nil

-- 3D recursive flood-fill across all connected ore blocks (costs 0 trash)
local function exploreOre3D()
    visited3D[cellKey(x, y, z)] = true
    minedCol[colKey(x, z)] = true

    if y > maxOreY then
        maxOreY = y
        bestUpElevator = {x = x, z = z, targetY = y}
    end
    if y < minOreY then
        minOreY = y
        bestDownElevator = {x = x, z = z, targetY = y}
    end

    -- 1. Check UP for ore
    local upK = cellKey(x, y + 1, z)
    if not visited3D[upK] and r.detectUp() and not r.compareUp() then
        if r.swingUp() and moveUp() then
            exploreOre3D()
            moveDown()
        end
    end

    -- 2. Check DOWN for ore
    local downK = cellKey(x, y - 1, z)
    if not visited3D[downK] and r.detectDown() and not r.compareDown() then
        if r.swingDown() and moveDown() then
            exploreOre3D()
            moveUp()
        end
    end

    -- 3. Check 4 horizontal neighbors for ore
    for d = 0, 3 do
        local dx = (d == 1 and 1) or (d == 3 and -1) or 0
        local dz = (d == 2 and 1) or (d == 0 and -1) or 0
        local hK = cellKey(x + dx, y, z + dz)

        if not visited3D[hK] then
            turnTo(d)
            if r.detect() and not r.compare() then
                if r.swing() and moveForward() then
                    exploreOre3D()
                    turnTo((d + 2) % 4)
                    moveForward()
                end
            end
        end
    end
end

-- Primary 2D plane exploration with connected ore flood-fill
local function explorePlane(skipSpokenColumns)
    local ck = colKey(x, z)
    planeVisited[ck] = true

    -- Fully mine any 3D connected ore component reachable from this cell for 0 trash
    exploreOre3D()

    for d = 0, 3 do
        local dx = (d == 1 and 1) or (d == 3 and -1) or 0
        local dz = (d == 2 and 1) or (d == 0 and -1) or 0
        local nCol = colKey(x + dx, z + dz)

        if not planeVisited[nCol] then
            turnTo(d)
            local canEnter = false

            if not r.detect() then
                -- Air: zero trash cost
                canEnter = true
            elseif not r.compare() then
                -- Ore: free to mine
                if r.swing() then
                    canEnter = true
                end
            else
                -- Trash block.
                -- If the column was already spoken for (its ore run mined from another height),
                -- we avoid digging trash in that column if instructed.
                if skipSpokenColumns and minedCol[nCol] then
                    canEnter = false
                else
                    if r.swing() then
                        canEnter = true
                    else
                        -- Bedrock boundary
                        planeVisited[nCol] = true
                    end
                end
            end

            if canEnter and moveForward() then
                explorePlane(skipSpokenColumns)
                turnTo((d + 2) % 4)
                moveForward()
            end
        end
    end
end

-- Navigate safely along already-cleared empty air cells on the base plane
local function goTo(targetX, targetZ)
    if x == targetX and z == targetZ then return true end
    local q = {{x = x, z = z}}
    local parent = {}
    local pvisited = {[colKey(x, z)] = true}

    while #q > 0 do
        local curr = table.remove(q, 1)
        if curr.x == targetX and curr.z == targetZ then break end

        for d = 0, 3 do
            local dx = (d == 1 and 1) or (d == 3 and -1) or 0
            local dz = (d == 2 and 1) or (d == 0 and -1) or 0
            local nx, nz = curr.x + dx, curr.z + dz
            local nk = colKey(nx, nz)

            if planeVisited[nk] and not pvisited[nk] then
                pvisited[nk] = true
                parent[nk] = {x = curr.x, z = curr.z, dir = d}
                table.insert(q, {x = nx, z = nz})
            end
        end
    end

    local path = {}
    local curr = {x = targetX, z = targetZ}
    while parent[colKey(curr.x, curr.z)] do
        local p = parent[colKey(curr.x, curr.z)]
        table.insert(path, 1, p.dir)
        curr = {x = p.x, z = p.z}
    end

    for _, d in ipairs(path) do
        turnTo(d)
        moveForward()
    end
    return true
end

-- Phase 1: Explore spawning plane (y = 0) while propagating 3D ore veins
explorePlane(true)

-- Phase 2: If a high-reaching ore vein formed a hollow vertical air shaft,
-- use it as a 0-trash elevator to catch upper disjoint veins without re-digging spoken columns
if bestUpElevator and bestUpElevator.targetY >= 4 then
    goTo(bestUpElevator.x, bestUpElevator.z)
    for _ = 1, bestUpElevator.targetY do moveUp() end

    explorePlane(true)

    for _ = 1, bestUpElevator.targetY do moveDown() end
    goTo(0, 0)
end

-- Phase 3: Check bottom disjoint veins through any deep downward hollow vein shaft
if bestDownElevator and bestDownElevator.targetY <= -4 then
    goTo(bestDownElevator.x, bestDownElevator.z)
    for _ = 1, -bestDownElevator.targetY do moveDown() end

    explorePlane(true)

    for _ = 1, -bestDownElevator.targetY do moveUp() end
    goTo(0, 0)
end



print('DONE')
while true do piccolo_yield() end
"#.as_bytes();

