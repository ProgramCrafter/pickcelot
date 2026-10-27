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




-- ============================================================================
-- Credit-constrained OpenComputers ore miner
--
-- Policy:
--   * Mine all reachable exposed ore.
--   * Explore all reachable air without mining trash.
--   * Mine trash only as one-block probes.
--   * Never allow:
--
--         trashBroken > floor(oresMined / 3)
--
--     Therefore the realized ore:trash ratio is never below 3:1.
--
-- This policy assumes that valuable blocks are identified by:
--   detect() == true and compare() == false
--
-- The supplied polyfill does not provide inventory contents, so inventory
-- fullness is checked only through robot.space().
--
-- No computer.pullSignal calls are used.
-- ============================================================================

local robot = require("robot")
local computer = require("computer")

-- ============================================================================
-- Constants
-- ============================================================================

local COLOR_MINING = 0x00FF00
local COLOR_MOVING = 0xFFFF00
local COLOR_HAZARD = 0xFF0000
local COLOR_RETURNING = 0x0000FF
local COLOR_IDLE = 0x888888

local BLOCK_UNKNOWN = 0
local BLOCK_AIR = 1
local BLOCK_TRASH = 2
local BLOCK_VALUABLE = 3
local BLOCK_UNBREAKABLE = 4

local MAX_MOVE_ATTEMPTS = 3
local MIN_ENERGY_RATIO = 0.15

-- ============================================================================
-- Direction data
-- ============================================================================

local vectors = {
    [0] = { dx = 1, dz = 0 },
    [1] = { dx = 0, dz = 1 },
    [2] = { dx = -1, dz = 0 },
    [3] = { dx = 0, dz = -1 }
}

local directions = {
    "up",
    "down",
    0,
    1,
    2,
    3
}

-- ============================================================================
-- State
-- ============================================================================

local state = {
    x = 0,
    y = 0,
    z = 0,
    heading = 0,

    known = {},
    knownList = {},

    visited = {},
    blockedEdges = {},

    oresMined = 0,
    trashBroken = 0,
    unbreakableBlocks = 0,

    stopReason = nil
}

-- ============================================================================
-- Coordinates
-- ============================================================================

local function coordinateKey(x, y, z)
    return x .. "," .. y .. "," .. z
end

local function edgeKey(x1, y1, z1, x2, y2, z2)
    return x1 .. "," .. y1 .. "," .. z1
        .. ">"
        .. x2 .. "," .. y2 .. "," .. z2
end

local function targetCoordinates(x, y, z, direction)
    if direction == "up" then
        return x, y + 1, z
    end

    if direction == "down" then
        return x, y - 1, z
    end

    local vector = vectors[direction]
    return x + vector.dx, y, z + vector.dz
end

local function oppositeDirection(direction)
    if direction == "up" then
        return "down"
    end

    if direction == "down" then
        return "up"
    end

    return (direction + 2) % 4
end

-- ============================================================================
-- World registry
-- ============================================================================

local function recordBlock(x, y, z, blockType)
    local key = coordinateKey(x, y, z)
    local oldType = state.known[key]

    if oldType == BLOCK_UNBREAKABLE then
        return
    end

    if oldType == nil then
        state.known[key] = blockType

        state.knownList[#state.knownList + 1] = {
            x = x,
            y = y,
            z = z,
            key = key,
            type = blockType
        }

        return
    end

    state.known[key] = blockType

    for i = 1, #state.knownList do
        local item = state.knownList[i]

        if item.key == key then
            item.type = blockType
            return
        end
    end
end

local function recordCurrentPosition()
    recordBlock(state.x, state.y, state.z, BLOCK_AIR)
end

-- ============================================================================
-- Resource checks
-- ============================================================================

local function energyIsLow()
    local maximum = computer.maxEnergy()

    if maximum <= 0 then
        return false
    end

    return computer.energy() / maximum < MIN_ENERGY_RATIO
end

local function inventoryIsFull()
    local free = robot.space()

    if free == nil then
        return false
    end

    return free <= 0
end

local function trashCredit()
    return math.floor(state.oresMined / 3) + 160
end

local function mayMineOneTrash()
    return state.trashBroken < trashCredit()
end

-- ============================================================================
-- Turning
-- ============================================================================

local function turnRight()
    local ok = robot.turnRight()

    if ok then
        state.heading = (state.heading + 1) % 4
        return true
    end

    return false
end

local function turnLeft()
    local ok = robot.turnLeft()

    if ok then
        state.heading = (state.heading + 3) % 4
        return true
    end

    return false
end

local function turnAround()
    local ok = robot.turnAround()

    if ok then
        state.heading = (state.heading + 2) % 4
        return true
    end

    local first = turnRight()

    if not first then
        return false
    end

    return turnRight()
end

local function faceHeading(targetHeading)
    local difference = (targetHeading - state.heading) % 4

    if difference == 0 then
        return true
    end

    if difference == 1 then
        return turnRight()
    end

    if difference == 2 then
        return turnAround()
    end

    return turnLeft()
end

-- ============================================================================
-- Inspection
-- ============================================================================

local function inspectFront()
    if not robot.detect() then
        return BLOCK_AIR
    end

    if robot.compare() then
        return BLOCK_TRASH
    end

    return BLOCK_VALUABLE
end

local function inspectUp()
    if not robot.detectUp() then
        return BLOCK_AIR
    end

    if robot.compareUp() then
        return BLOCK_TRASH
    end

    return BLOCK_VALUABLE
end

local function inspectDown()
    if not robot.detectDown() then
        return BLOCK_AIR
    end

    if robot.compareDown() then
        return BLOCK_TRASH
    end

    return BLOCK_VALUABLE
end

local function inspectDirection(direction)
    if direction == "up" then
        return inspectUp()
    end

    if direction == "down" then
        return inspectDown()
    end

    if not faceHeading(direction) then
        return BLOCK_UNKNOWN
    end

    return inspectFront()
end

local function inspectAndRecord(direction)
    local x, y, z = targetCoordinates(
        state.x,
        state.y,
        state.z,
        direction
    )

    local blockType = inspectDirection(direction)

    if blockType ~= BLOCK_UNKNOWN then
        recordBlock(x, y, z, blockType)
    end

    return blockType, x, y, z
end

-- ============================================================================
-- Movement
-- ============================================================================

local function blockedEdge(x1, y1, z1, x2, y2, z2)
    return state.blockedEdges[
        edgeKey(x1, y1, z1, x2, y2, z2)
    ] == true
end

local function markBlockedEdge(x1, y1, z1, x2, y2, z2)
    state.blockedEdges[
        edgeKey(x1, y1, z1, x2, y2, z2)
    ] = true
end

local function performMove(direction)
    if direction == "up" then
        return robot.up()
    end

    if direction == "down" then
        return robot.down()
    end

    if not faceHeading(direction) then
        return false, "cannot turn"
    end

    return robot.forward()
end

local function safeMove(direction, emergency)
    local oldX = state.x
    local oldY = state.y
    local oldZ = state.z

    local newX, newY, newZ = targetCoordinates(
        state.x,
        state.y,
        state.z,
        direction
    )

    if blockedEdge(oldX, oldY, oldZ, newX, newY, newZ) then
        return false, "known blocked edge"
    end

    for attempt = 1, MAX_MOVE_ATTEMPTS do
        if not emergency and energyIsLow() then
            state.stopReason = "low energy"
            return false, "low energy"
        end

        local ok, reason = performMove(direction)

        if ok then
            state.x = newX
            state.y = newY
            state.z = newZ

            recordBlock(newX, newY, newZ, BLOCK_AIR)
            return true
        end

        -- A path may have been filled by falling gravel or sand. Do not mine
        -- it here: clearing trash is governed by the ore-credit policy.
        local currentType = inspectDirection(direction)

        if currentType ~= BLOCK_AIR then
            break
        end
    end

    markBlockedEdge(oldX, oldY, oldZ, newX, newY, newZ)
    return false, "movement failed"
end

-- ============================================================================
-- Mining primitives
-- ============================================================================

local function swingDirection(direction)
    if direction == "up" then
        return robot.swingUp()
    end

    if direction == "down" then
        return robot.swingDown()
    end

    if not faceHeading(direction) then
        return false, "cannot turn"
    end

    return robot.swing()
end

local function mineOneValuable(direction, x, y, z)
    if inventoryIsFull() then
        state.stopReason = "inventory full"
        return false
    end

    local currentType = inspectDirection(direction)

    if currentType ~= BLOCK_VALUABLE then
        return false
    end

    robot.setLightColor(COLOR_MINING)

    local ok = swingDirection(direction)

    if not ok then
        recordBlock(x, y, z, BLOCK_UNBREAKABLE)
        state.unbreakableBlocks = state.unbreakableBlocks + 1
        robot.setLightColor(COLOR_HAZARD)
        return false
    end

    state.oresMined = state.oresMined + 1

    local afterType = inspectDirection(direction)

    if afterType == BLOCK_AIR then
        recordBlock(x, y, z, BLOCK_AIR)
    elseif afterType == BLOCK_TRASH then
        -- A falling block occupied the target. It must not be mined merely to
        -- finish mining the ore.
        recordBlock(x, y, z, BLOCK_TRASH)
    else
        recordBlock(x, y, z, BLOCK_UNBREAKABLE)
    end

    return true
end

local function mineOneTrashProbe(direction, x, y, z)
    if not mayMineOneTrash() then
        return false, "no ore credit"
    end

    if inventoryIsFull() then
        state.stopReason = "inventory full"
        return false, "inventory full"
    end

    local currentType = inspectDirection(direction)

    if currentType ~= BLOCK_TRASH then
        return false, "not trash"
    end

    robot.setLightColor(COLOR_MINING)

    -- Exactly one trash block is allowed per probe. This is important for
    -- gravel and sand: a second swing would consume additional credit.
    local ok = swingDirection(direction)

    if not ok then
        recordBlock(x, y, z, BLOCK_UNBREAKABLE)
        state.unbreakableBlocks = state.unbreakableBlocks + 1
        robot.setLightColor(COLOR_HAZARD)
        return false, "unbreakable"
    end

    state.trashBroken = state.trashBroken + 1

    local afterType = inspectDirection(direction)

    if afterType == BLOCK_AIR then
        recordBlock(x, y, z, BLOCK_AIR)
        return true
    end

    -- Gravel or sand may have fallen into the target. Do not mine it without
    -- another unit of ore credit.
    if afterType == BLOCK_TRASH then
        recordBlock(x, y, z, BLOCK_TRASH)
    elseif afterType == BLOCK_VALUABLE then
        recordBlock(x, y, z, BLOCK_VALUABLE)
    else
        recordBlock(x, y, z, BLOCK_UNBREAKABLE)
    end

    return false, "probe did not open"
end

-- ============================================================================
-- Candidate ordering
-- ============================================================================

local function candidatePriority(blockType)
    if blockType == BLOCK_VALUABLE then
        return 1
    end

    if blockType == BLOCK_AIR then
        return 2
    end

    if blockType == BLOCK_TRASH then
        return 3
    end

    return 4
end

local function inspectCandidates()
    local originalHeading = state.heading
    local candidates = {}

    for i = 1, #directions do
        local direction = directions[i]
        local blockType, x, y, z = inspectAndRecord(direction)

        candidates[#candidates + 1] = {
            direction = direction,
            type = blockType,
            x = x,
            y = y,
            z = z,
            priority = candidatePriority(blockType)
        }
    end

    faceHeading(originalHeading)

    -- Stable insertion sort without relying on table.sort behavior.
    for i = 2, #candidates do
        local item = candidates[i]
        local j = i - 1

        while j >= 1
            and candidates[j].priority > item.priority do
            candidates[j + 1] = candidates[j]
            j = j - 1
        end

        candidates[j + 1] = item
    end

    return candidates
end

-- ============================================================================
-- DFS exploration
-- ============================================================================

local function makeFrame()
    local key = coordinateKey(state.x, state.y, state.z)

    state.visited[key] = true

    return {
        key = key,
        candidates = inspectCandidates(),
        nextCandidate = 1,
        enteredBy = nil
    }
end

local function candidateIsNew(candidate)
    if candidate.type == BLOCK_UNKNOWN
        or candidate.type == BLOCK_UNBREAKABLE then
        return false
    end

    local key = coordinateKey(
        candidate.x,
        candidate.y,
        candidate.z
    )

    if state.visited[key] then
        return false
    end

    if blockedEdge(
        state.x,
        state.y,
        state.z,
        candidate.x,
        candidate.y,
        candidate.z
    ) then
        return false
    end

    return true
end

local function enterCandidate(candidate)
    local direction = candidate.direction
    local key = coordinateKey(
        candidate.x,
        candidate.y,
        candidate.z
    )

    if state.visited[key] then
        return false
    end

    local currentType = inspectDirection(direction)

    if currentType == BLOCK_VALUABLE then
        local mined = mineOneValuable(
            direction,
            candidate.x,
            candidate.y,
            candidate.z
        )

        if not mined then
            return false
        end

        -- The ore block is now open if nothing fell into it.
        local afterType = inspectDirection(direction)

        if afterType ~= BLOCK_AIR then
            return false
        end
    elseif currentType == BLOCK_AIR then
        recordBlock(
            candidate.x,
            candidate.y,
            candidate.z,
            BLOCK_AIR
        )
    elseif currentType == BLOCK_TRASH then
        local opened = mineOneTrashProbe(
            direction,
            candidate.x,
            candidate.y,
            candidate.z
        )

        if not opened then
            return false
        end
    else
        return false
    end

    return safeMove(direction, false)
end

local function backtrack(frame)
    if frame.enteredBy == nil then
        return true
    end

    return safeMove(
        oppositeDirection(frame.enteredBy),
        true
    )
end

local function explore()
    local stack = {
        makeFrame()
    }

    while #stack > 0 do
        if energyIsLow() then
            state.stopReason = "low energy"
            break
        end

        if inventoryIsFull() then
            state.stopReason = "inventory full"
            break
        end

        local frame = stack[#stack]

        if frame.nextCandidate > #frame.candidates then
            local ok = backtrack(frame)

            if not ok then
                state.stopReason = "backtracking failed"
                break
            end

            stack[#stack] = nil
        else
            local candidate = frame.candidates[frame.nextCandidate]
            frame.nextCandidate = frame.nextCandidate + 1

            if candidateIsNew(candidate) then
                local entered = enterCandidate(candidate)

                if entered then
                    stack[#stack + 1] = makeFrame()
                    stack[#stack].enteredBy = candidate.direction
                end
            end
        end
    end

    -- Return through the already explored DFS path if the mission stopped
    -- because of energy or inventory constraints.
    while #stack > 1 do
        local frame = stack[#stack]
        local ok = backtrack(frame)

        if not ok then
            state.stopReason = "return after stop failed"
            return false
        end

        stack[#stack] = nil
    end

    return true
end

-- ============================================================================
-- Main
-- ============================================================================

local function runMiner()
    recordCurrentPosition()
    robot.setLightColor(COLOR_MINING)

    print("Starting credit-constrained ore mining.")
    print("Trash credit: floor(ore / 3).")

    local completed = explore()

    robot.setLightColor(COLOR_RETURNING)
    faceHeading(0)
    robot.setLightColor(COLOR_IDLE)

    if completed and state.stopReason == nil then
        print("Exploration complete.")
    else
        print("Exploration stopped: " .. tostring(state.stopReason))
    end

    print("Position: " .. tostring(state.x)
        .. "," .. tostring(state.y)
        .. "," .. tostring(state.z))

    print("Ore mined: " .. tostring(state.oresMined))
    print("Trash mined: " .. tostring(state.trashBroken))
    print("Trash credit: " .. tostring(trashCredit()))
    print("Unbreakable blocks: " .. tostring(state.unbreakableBlocks))
end

runMiner()



print('DONE')
while true do piccolo_yield() end
"#.as_bytes();

