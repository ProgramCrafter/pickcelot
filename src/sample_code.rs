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

do
    local r = require('robot')

    local PLANE_STEP = 4

    -- State encapsulated entirely within closure
    local state = {
        x = 0,
        y = 0,
        z = 0,
        facing = 0, -- 0 = North (-z), 1 = East (+x), 2 = South (+z), 3 = West (-x)
        visited3D = {},
        minedCol = {},
        clearedOnPlane = {}
    }

    local function cellKey(cx, cy, cz)
        return cx .. ":" .. cy .. ":" .. cz
    end

    local function colKey(cx, cz)
        return cx .. ":" .. cz
    end

    local function turnTo(target)
        local diff = (target - state.facing) % 4
        if diff == 1 then
            r.turnRight()
        elseif diff == 2 then
            r.turnAround()
        elseif diff == 3 then
            r.turnLeft()
        end
        state.facing = target
    end

    local function moveForward()
        if r.forward() then
            if state.facing == 0 then state.z = state.z - 1
            elseif state.facing == 1 then state.x = state.x + 1
            elseif state.facing == 2 then state.z = state.z + 1
            elseif state.facing == 3 then state.x = state.x - 1 end
            return true
        end
        return false
    end

    local function moveUp()
        if r.up() then
            state.y = state.y + 1
            return true
        end
        return false
    end

    local function moveDown()
        if r.down() then
            state.y = state.y - 1
            return true
        end
        return false
    end

    -- BFS pathfinder strictly across already cleared air cells on plane Y
    local function goToOnPlane(targetX, targetZ, planeY)
        if state.x == targetX and state.z == targetZ then return true end

        local cleared = state.clearedOnPlane[planeY] or {}
        local q = {{x = state.x, z = state.z}}
        local parent = {}
        local pvisited = {[colKey(state.x, state.z)] = true}

        while #q > 0 do
            local curr = table.remove(q, 1)
            if curr.x == targetX and curr.z == targetZ then break end

            for d = 0, 3 do
                local dx = (d == 1 and 1) or (d == 3 and -1) or 0
                local dz = (d == 2 and 1) or (d == 0 and -1) or 0
                local nx, nz = curr.x + dx, curr.z + dz
                local nk = colKey(nx, nz)

                if cleared[nk] and not pvisited[nk] then
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
            if not moveForward() then return false end
        end
        return true
    end

    -- 3D flood-fill through connected ore blocks (costs 0 trash)
    local function exploreOre3D(stats)
        local ck = cellKey(state.x, state.y, state.z)
        state.visited3D[ck] = true
        state.minedCol[colKey(state.x, state.z)] = true

        if stats then
            if state.y > stats.maxY then
                stats.maxY = state.y
                stats.bestUpX = state.x
                stats.bestUpZ = state.z
            end
            if state.y < stats.minY then
                stats.minY = state.y
                stats.bestDownX = state.x
                stats.bestDownZ = state.z
            end
        end

        -- Check UP for ore
        local upK = cellKey(state.x, state.y + 1, state.z)
        if not state.visited3D[upK] and r.detectUp() and not r.compareUp() then
            if r.swingUp() and moveUp() then
                exploreOre3D(stats)
                moveDown()
            end
        end

        -- Check DOWN for ore
        local downK = cellKey(state.x, state.y - 1, state.z)
        if not state.visited3D[downK] and r.detectDown() and not r.compareDown() then
            if r.swingDown() and moveDown() then
                exploreOre3D(stats)
                moveUp()
            end
        end

        -- Check 4 horizontal neighbors for ore
        for d = 0, 3 do
            local dx = (d == 1 and 1) or (d == 3 and -1) or 0
            local dz = (d == 2 and 1) or (d == 0 and -1) or 0
            local hK = cellKey(state.x + dx, state.y, state.z + dz)

            if not state.visited3D[hK] then
                turnTo(d)
                if r.detect() and not r.compare() then
                    if r.swing() and moveForward() then
                        exploreOre3D(stats)
                        turnTo((d + 2) % 4)
                        moveForward()
                    end
                end
            end
        end
    end

    -- 2D DFS exploration of plane Y, skipping columns whose ore runs were already mined
    local function explorePlane(planeY, stats)
        if not state.clearedOnPlane[planeY] then
            state.clearedOnPlane[planeY] = {}
        end

        local ck = colKey(state.x, state.z)
        state.clearedOnPlane[planeY][ck] = true

        -- Flood-fill all 3D connected ore reachable from here
        exploreOre3D(stats)

        for d = 0, 3 do
            local dx = (d == 1 and 1) or (d == 3 and -1) or 0
            local dz = (d == 2 and 1) or (d == 0 and -1) or 0
            local nCol = colKey(state.x + dx, state.z + dz)

            if not state.clearedOnPlane[planeY][nCol] then
                turnTo(d)
                local canEnter = false

                if not r.detect() then
                    -- Empty air
                    canEnter = true
                elseif not r.compare() then
                    -- Ore block
                    if r.swing() then
                        canEnter = true
                    end
                else
                    -- Trash block
                    -- If this column's ore was already spoken for, do not dig trash here
                    if not state.minedCol[nCol] then
                        if r.swing() then
                            canEnter = true
                        else
                            -- Solid bedrock boundary
                            state.clearedOnPlane[planeY][nCol] = true
                        end
                    end
                end

                if canEnter and moveForward() then
                    explorePlane(planeY, stats)
                    turnTo((d + 2) % 4)
                    moveForward()
                end
            end
        end
    end

    -- Ascend towards targetY. Follows air shafts where available,
    -- and digs through trash if a full elevator is not found.
    local function ascendTo(targetY)
        local moved = 0
        while state.y < targetY do
            if not r.detectUp() then
                if not moveUp() then break end
                moved = moved + 1
            else
                if not r.swingUp() then break end -- bedrock boundary reached
                if not moveUp() then break end
                moved = moved + 1
            end
        end
        return moved
    end

    -- Descend towards targetY. Follows air shafts where available,
    -- and digs through trash if a full elevator is not found.
    local function descendTo(targetY)
        local moved = 0
        while state.y > targetY do
            if not r.detectDown() then
                if not moveDown() then break end
                moved = moved + 1
            else
                if not r.swingDown() then break end -- bedrock boundary reached
                if not moveDown() then break end
                moved = moved + 1
            end
        end
        return moved
    end

    -- Recursive multi-stratum explorer (supports arbitrarily large heights)
    local function exploreStratum(planeY, canAscend, canDescend)
        local stats = {
            maxY = planeY,
            bestUpX = state.x,
            bestUpZ = state.z,
            minY = planeY,
            bestDownX = state.x,
            bestDownZ = state.z
        }

        local startX, startZ = state.x, state.z

        -- 1. Explore current horizontal plane and harvest all connected ore runs
        explorePlane(planeY, stats)

        -- 2. Recursively ascend to upper plane
        if canAscend then
            local nextUpY = planeY + PLANE_STEP
            local elevatorX = stats.bestUpX or startX
            local elevatorZ = stats.bestUpZ or startZ

            goToOnPlane(elevatorX, elevatorZ, planeY)

            local climbed = ascendTo(nextUpY)
            if state.y > planeY then
                exploreStratum(state.y, true, false)
                for _ = 1, climbed do moveDown() end
            end

            goToOnPlane(startX, startZ, planeY)
        end

        -- 3. Recursively descend to lower plane
        if canDescend then
            local nextDownY = planeY - PLANE_STEP
            local elevatorX = stats.bestDownX or startX
            local elevatorZ = stats.bestDownZ or startZ

            goToOnPlane(elevatorX, elevatorZ, planeY)

            local dropped = descendTo(nextDownY)
            if state.y < planeY then
                exploreStratum(state.y, false, true)
                for _ = 1, dropped do moveUp() end
            end

            goToOnPlane(startX, startZ, planeY)
        end
    end

    -- Start recursive exploration from initial spawning position
    exploreStratum(0, true, true)
end




print('DONE')
while true do piccolo_yield() end
"#.as_bytes();

