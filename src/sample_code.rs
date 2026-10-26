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
-- Exhaustive OpenComputers ore miner
--
-- The robot explores the reachable connected region using iterative DFS.
--
-- Block classification follows the supplied polyfill:
--   detect() == false                  -> air
--   detect() == true and compare()    -> trash
--   detect() == true and compare()==false -> valuable or unbreakable
--
-- Coordinates:
--   x: horizontal axis
--   y: vertical axis
--   z: horizontal axis
--
-- Headings:
--   0 = +x
--   1 = +z
--   2 = -x
--   3 = -z
-- ============================================================================

local robot = require("robot")
local computer = require("computer")

-- ============================================================================
-- Constants
-- ============================================================================

local COLOR_NORMAL = 0x00FF00
local COLOR_MOVE = 0xFFFF00
local COLOR_HAZARD = 0xFF0000
local COLOR_RETURN = 0x0000FF
local COLOR_IDLE = 0x888888

local BLOCK_UNKNOWN = 0
local BLOCK_AIR = 1
local BLOCK_TRASH = 2
local BLOCK_VALUABLE = 3
local BLOCK_UNBREAKABLE = 4

local MAX_SWINGS_PER_TARGET = 64
local MAX_MOVE_ATTEMPTS = 3
local MIN_ENERGY_RATIO = 0.15

-- ============================================================================
-- Direction data
-- ============================================================================

local headingVector = {
    [0] = { dx = 1, dz = 0 },
    [1] = { dx = 0, dz = 1 },
    [2] = { dx = -1, dz = 0 },
    [3] = { dx = 0, dz = -1 }
}

local horizontalDirections = { 0, 1, 2, 3 }

local allDirections = {
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

    -- known[key] = block type
    known = {},

    -- knownList avoids parsing coordinate keys.
    knownList = {},

    -- visited[key] = true for positions actually entered.
    visited = {},

    -- impassableEdges[key] = true for failed movements.
    impassableEdges = {},

    oresMined = 0,
    trashBroken = 0,
    failedTargets = 0,

    stopReason = nil
}

-- ============================================================================
-- Coordinate helpers
-- ============================================================================

local function coordinateKey(x, y, z)
    return x .. "," .. y .. "," .. z
end

local function edgeKey(x1, y1, z1, x2, y2, z2)
    return x1 .. "," .. y1 .. "," .. z1
        .. ">"
        .. x2 .. "," .. y2 .. "," .. z2
end

local function targetForDirection(x, y, z, direction)
    if direction == "up" then
        return x, y + 1, z
    end

    if direction == "down" then
        return x, y - 1, z
    end

    local vector = headingVector[direction]
    return x + vector.dx, y, z + vector.dz
end

local function inverseDirection(direction)
    if direction == "up" then
        return "down"
    end

    if direction == "down" then
        return "up"
    end

    return (direction + 2) % 4
end

-- ============================================================================
-- Known-world registry
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
-- Energy and inventory checks
-- ============================================================================

local function energyIsLow()
    local maximum = computer.maxEnergy()

    if maximum <= 0 then
        return false
    end

    return computer.energy() / maximum < MIN_ENERGY_RATIO
end

local function inventoryIsFull()
    -- The supplied polyfill returns math.huge here.
    local free = robot.space()
    return free ~= nil and free <= 0
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

    -- Fallback for implementations without hidden$turnaround.
    local first = turnRight()
    local second = false

    if first then
        second = turnRight()
    end

    return first and second
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

local function classifyFront()
    if not robot.detect() then
        return BLOCK_AIR
    end

    if robot.compare() then
        return BLOCK_TRASH
    end

    return BLOCK_VALUABLE
end

local function classifyUp()
    if not robot.detectUp() then
        return BLOCK_AIR
    end

    if robot.compareUp() then
        return BLOCK_TRASH
    end

    return BLOCK_VALUABLE
end

local function classifyDown()
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
        return classifyUp()
    end

    if direction == "down" then
        return classifyDown()
    end

    if not faceHeading(direction) then
        return BLOCK_UNKNOWN
    end

    return classifyFront()
end

local function inspectAndRecord(direction)
    local x, y, z = targetForDirection(
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
-- Block breaking
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

local function clearDirection(direction)
    local x, y, z = targetForDirection(
        state.x,
        state.y,
        state.z,
        direction
    )

    local swings = 0

    while swings < MAX_SWINGS_PER_TARGET do
        local blockType = inspectDirection(direction)

        if blockType == BLOCK_AIR then
            recordBlock(x, y, z, BLOCK_AIR)
            return true
        end

        if blockType == BLOCK_UNKNOWN then
            return false, "inspection failed"
        end

        if inventoryIsFull() then
            state.stopReason = "inventory full"
            return false, "inventory full"
        end

        local ok = swingDirection(direction)

        if not ok then
            recordBlock(x, y, z, BLOCK_UNBREAKABLE)
            state.failedTargets = state.failedTargets + 1
            robot.setLightColor(COLOR_HAZARD)
            return false, "unbreakable"
        end

        swings = swings + 1

        if blockType == BLOCK_VALUABLE then
            state.oresMined = state.oresMined + 1
        elseif blockType == BLOCK_TRASH then
            state.trashBroken = state.trashBroken + 1
        end

        -- Reinspection on the next iteration handles gravel and sand that
        -- fall into the cleared position.
    end

    recordBlock(x, y, z, BLOCK_UNBREAKABLE)
    state.failedTargets = state.failedTargets + 1

    robot.setLightColor(COLOR_HAZARD)
    return false, "too many falling blocks"
end

-- ============================================================================
-- Movement
-- ============================================================================

local function movementTarget(direction)
    return targetForDirection(
        state.x,
        state.y,
        state.z,
        direction
    )
end

local function markImpassable(x1, y1, z1, x2, y2, z2)
    state.impassableEdges[
        edgeKey(x1, y1, z1, x2, y2, z2)
    ] = true
end

local function isImpassable(x1, y1, z1, x2, y2, z2)
    return state.impassableEdges[
        edgeKey(x1, y1, z1, x2, y2, z2)
    ] == true
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

local function safeMove(direction)
    local oldX = state.x
    local oldY = state.y
    local oldZ = state.z

    local newX, newY, newZ = movementTarget(direction)

    if isImpassable(oldX, oldY, oldZ, newX, newY, newZ) then
        return false, "known impassable edge"
    end

    for attempt = 1, MAX_MOVE_ATTEMPTS do
        if energyIsLow() then
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

        -- The path may have been filled by falling gravel or sand.
        local blockType = inspectDirection(direction)

        if blockType ~= BLOCK_AIR and blockType ~= BLOCK_UNKNOWN then
            clearDirection(direction)
        end
    end

    markImpassable(oldX, oldY, oldZ, newX, newY, newZ)
    return false, "movement failed"
end

-- ============================================================================
-- Frontier ordering
-- ============================================================================

local function blockPriority(blockType)
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

local function inspectDirectionsInPriorityOrder()
    local candidates = {}
    local originalHeading = state.heading

    for i = 1, #allDirections do
        local direction = allDirections[i]
        local blockType, x, y, z = inspectAndRecord(direction)

        candidates[#candidates + 1] = {
            direction = direction,
            type = blockType,
            x = x,
            y = y,
            z = z,
            priority = blockPriority(blockType)
        }
    end

    faceHeading(originalHeading)

    -- Stable insertion sort. Valuable blocks are handled first, followed by
    -- already-open space, then trash frontier blocks.
    for i = 2, #candidates do
        local current = candidates[i]
        local j = i - 1

        while j >= 1
            and candidates[j].priority > current.priority do
            candidates[j + 1] = candidates[j]
            j = j - 1
        end

        candidates[j + 1] = current
    end

    return candidates
end

-- ============================================================================
-- DFS frame handling
-- ============================================================================

local function makeFrame()
    local key = coordinateKey(state.x, state.y, state.z)

    state.visited[key] = true

    return {
        x = state.x,
        y = state.y,
        z = state.z,
        key = key,
        candidates = inspectDirectionsInPriorityOrder(),
        nextCandidate = 1,
        enteredBy = nil
    }
end

local function candidateCanBeEntered(candidate)
    if candidate.type == BLOCK_UNBREAKABLE
        or candidate.type == BLOCK_UNKNOWN then
        return false
    end

    local targetKey = coordinateKey(
        candidate.x,
        candidate.y,
        candidate.z
    )

    if state.visited[targetKey] then
        return false
    end

    if isImpassable(
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

local function openCandidate(candidate)
    local direction = candidate.direction
    local targetKey = coordinateKey(
        candidate.x,
        candidate.y,
        candidate.z
    )

    local currentType = inspectDirection(direction)

    if currentType == BLOCK_UNKNOWN
        or currentType == BLOCK_UNBREAKABLE then
        return false
    end

    if state.visited[targetKey] then
        return false
    end

    if currentType ~= BLOCK_AIR then
        local cleared = clearDirection(direction)

        if not cleared then
            return false
        end

        recordBlock(
            candidate.x,
            candidate.y,
            candidate.z,
            BLOCK_AIR
        )
    end

    return safeMove(direction)
end

local function returnToParent(frame)
    if frame.enteredBy == nil then
        return true
    end

    local direction = inverseDirection(frame.enteredBy)

    return safeMove(direction)
end

-- ============================================================================
-- Exhaustive traversal
-- ============================================================================

local function traverse()
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
            local returned = returnToParent(frame)

            if not returned then
                state.stopReason = "could not backtrack"
                break
            end

            stack[#stack] = nil
        else
            local candidate = frame.candidates[frame.nextCandidate]
            frame.nextCandidate = frame.nextCandidate + 1

            if candidateCanBeEntered(candidate) then
                local opened = openCandidate(candidate)

                if opened then
                    robot.setLightColor(COLOR_NORMAL)

                    stack[#stack + 1] = makeFrame()
                    stack[#stack].enteredBy = candidate.direction
                end
            end
        end
    end

    -- If traversal stopped early, unwind all entered frames. This returns the
    -- robot to the starting coordinate whenever the already-used path remains
    -- traversable.
    while #stack > 1 do
        local frame = stack[#stack]
        local returned = returnToParent(frame)

        if not returned then
            state.stopReason = "could not return after abort"
            return false
        end

        stack[#stack] = nil
    end

    return true
end

-- ============================================================================
-- Main program
-- ============================================================================

local function runMiner()
    recordCurrentPosition()
    robot.setLightColor(COLOR_NORMAL)

    print("Beginning exhaustive ore traversal.")

    local completed = traverse()

    if completed and state.stopReason == nil then
        print("Traversal complete.")
    else
        print("Traversal stopped: " .. tostring(state.stopReason))
    end

    -- The traversal should already have returned to the origin. Restore the
    -- original heading as a final safety measure.
    robot.setLightColor(COLOR_RETURN)
    faceHeading(0)
    robot.setLightColor(COLOR_IDLE)

    print(
        "Position: ("
        .. tostring(state.x)
        .. ","
        .. tostring(state.y)
        .. ","
        .. tostring(state.z)
        .. ")"
    )

    print("Ores mined: " .. tostring(state.oresMined))
    print("Trash blocks mined: " .. tostring(state.trashBroken))
    print("Unbreakable targets: " .. tostring(state.failedTargets))
end

runMiner()





print('DONE')
while true do piccolo_yield() end
"#.as_bytes();

