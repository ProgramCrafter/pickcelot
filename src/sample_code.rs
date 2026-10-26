pub const LUA_CODE: &[u8] = r#"
-- Polyfills

local print
local require

local do_init
do_init = function()
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
    print = function(s)
        print_y = print_y + 1
        if print_y > print_h then print_y = 1 end
        piccolo_component_invoke('$gpu', 'gpu.set', 1, print_y, s, false)
    end
    
    tonumber = tonumber or function() return nil end
    
    do_init = nil
end
do_init()

-- Totoro Recursive Miner  0.5H --
-- computercraft.ru (c) 01.2015 --
local robot = require('robot')
local event = require('event')
local sides = require('sides')
local computer = require('computer')
local com = require('component')

function trytoload(name)
  if com.isAvailable(name) then
    return com.getPrimary(name)
  else
    return nil
  end
end

local gen = trytoload("generator")
local cloader = trytoload("chunkloader")

TECH_SLOTS = 3
VANILLA_CHEST = true
PATHWAYS = false
DROP_TRASH = false
DEAD_END = 30
USE_CLOADER = true

INV_SIZE = robot.inventorySize()
if VANILLA_CHEST then INV_SIZE = math.min(INV_SIZE, TECH_SLOTS + 27) end
MAX = 1024

-- statictics
local moves = 0

local light_allowed = (robot.setLightColor ~= nil)
local color = {}
color.working = 0x00ff00
color.idle = 0xff0000
color.warning = 0xffff10

local trash_slots = TECH_SLOTS - 1
local chest_slot = TECH_SLOTS
local empty_slot = TECH_SLOTS + 1

local loc = {x=0, y=0, z=0, d=0}
local lode = {}

local a = {...}
local l = tonumber(a[1]) or 22
local w = tonumber(a[2]) or 17
local comeback = a[3] or 'false'
local halfw = math.floor(w/2)

if a[1] == '?' or a[1] == '-h' or a[1] == '--help' then
  print("Синтаксис: mine <length> [width] [comeback]")
  return
end
if gen == nil then 
  print([[[WARNING] Генератор не обнаружен!\n
    Робот не проработает долго без подзарядки.]])
end
if cloader ~= nil then 
  print([[[WARNING] Обнаружен чанклодер!\n
    На сервере ComputerCraft IT 1.7.10 его использование 
    приведет к моментальной разрядке батареи!]])
end

-- ============================= N A V I G A T I O N ============================= --
function forward()
  if robot.forward() then
    if loc.d == 0 then loc.y = loc.y+1
    elseif loc.d == 1 then loc.x = loc.x+1
    elseif loc.d == 2 then loc.y = loc.y-1
    else loc.x = loc.x-1 end
    moves = moves + 1
    return true
  else
    return false
  end
end
function back()
  if robot.back() then
    if loc.d == 0 then loc.y = loc.y-1
    elseif loc.d == 1 then loc.x = loc.x-1
    elseif loc.d == 2 then loc.y = loc.y+1
    else loc.x = loc.x+1 end
    moves = moves + 1
    return true
  else
    return false
  end
end
function up()
  if robot.up() then
    loc.z = loc.z+1
    moves = moves + 1
    return true
  else
    return false
  end
end
function down()
  if robot.down() then
    loc.z = loc.z-1
    moves = moves + 1
    return true
  else
    return false
  end
end
function turnRight()
  loc.d = (loc.d+1)%4
  robot.turnRight()
end
function turnAround()
  loc.d = (loc.d+2)%4
  robot.turnAround()
end
function turnLeft()
  loc.d = (loc.d+3)%4
  robot.turnLeft()
end

-- ========================== F U E L  C O N T R O L ============================= --
function check_fuel()
  if gen ~= nil then
    if (computer.maxEnergy() - computer.energy()) > 1000 then
      if gen.count() == 0 then
        for i=empty_slot, INV_SIZE do
          robot.select(i)
          if gen.insert() then break end
        end
      end
    end
  end
end

-- ========================== I T E M  C O N T R O L ============================= --
function check_loot()
  for i=empty_slot, INV_SIZE do
    if robot.count(i) == 0 then return false end
  end
  return true
end

function unload()
  -- place for chest
  robot.select(chest_slot)
  while robot.swing() do end
  robot.swingUp()
  up()
  while robot.swing() do end
  down()
  if robot.place() then
    -- put items
    for i=empty_slot, INV_SIZE do
      robot.select(i)
      while robot.drop() do end
    end

    -- grab ender chest
    if not VANILLA_CHEST then
      robot.select(chest_slot)
      robot.swing()
    end
  end
end

function check_trash()
  for i=1, trash_slots do
    -- if too many trash of this type
    if robot.space(i) == 0 then
      -- drop all but one in trash slot
      robot.select(i)
      robot.dropDown(robot.count(i)-1)
      -- and drop all others
      for j=empty_slot, INV_SIZE do
        if robot.compareTo(j) then
          robot.select(j)
          robot.dropDown()
          robot.select(i)
        end
      end
    end
  end
end

-- ================================ M I N I N G ================================== --
function qforward()
  -- while not forward() do robot.swing() end
  while not forward() and robot.swing() do end
end

function trash(side)
  for i=1, trash_slots do
    robot.select(i)
    if side == sides.up then
      a,t = robot.detectUp()
      if t ~= 'solid' or robot.compareUp() then return true end
    elseif side == sides.front then
      a,t = robot.detect()
      if t ~= 'solid' or robot.compare() then return true end
    else
      a,t = robot.detectDown()
      if t ~= 'solid' or robot.compareDown() then return true end
    end
  end
  return false
end

function mine(side)
  local direct = loc.d
  local backdir = (direct+2)%4

  if side == sides.up then
    c = 0
    while not up() do
      robot.swingUp()
      c = c + 1
      if c>DEAD_END then return end
    end
  elseif side == sides.front then
    c = 0
    while not forward() do
      robot.swing()
      c = c + 1
      if c>DEAD_END then return end
    end
  elseif side == sides.down then
    c = 0
    while not down() do
      robot.swingDown()
      c = c + 1
      if c>DEAD_END then return end
    end
  end

  lode[loc.x*MAX*MAX + loc.y*MAX + loc.z] = true

  -- check further direction
  if lode[loc.x*MAX*MAX + loc.y*MAX + (loc.z+1)] == nil then
    if not trash(sides.up) then mine(sides.up)
    else lode[loc.x*MAX*MAX + loc.y*MAX + (loc.z+1)] = false end
  end
  if lode[loc.x*MAX*MAX + loc.y*MAX + (loc.z-1)] == nil then
    if not trash(sides.down) then mine(sides.down)
    else lode[loc.x*MAX*MAX + loc.y*MAX + (loc.z-1)] = false end
  end

  for i=loc.d, loc.d+3 do
    local a = i%4
    local x = loc.x
    local y = loc.y
    if a == 0 then y = y + 1
    elseif a == 1 then x = x + 1
    elseif a == 2 then y = y - 1
    else x = x - 1 end
    if lode[x*MAX*MAX + y*MAX + loc.z] == nil then
      while loc.d < a do turnRight() end
      while loc.d > a do turnLeft() end
      if not trash(sides.front) then mine(sides.front) 
      else lode[x*MAX*MAX + y*MAX + loc.z] = false end
    end
  end

  -- come back
  if side == sides.up then
    down()
  elseif side == sides.front then
    if loc.d == direct and back() then
      -- yet ready =)
    else
      while loc.d < backdir do turnRight() end
      while loc.d > backdir do turnLeft() end
      qforward()
    end
  elseif side == sides.down then
    up()
  end
end

function go(side)
  direct = loc.d
  lode = {}
  mine(side)
  while loc.d < direct do turnRight() end
  while loc.d > direct do turnLeft() end
end

function step(progress)
  -- every tenth
  tenth = (loc.x % 15 == 0)
  -- dig one row
  for x=1, w do
    -- check up/down
    if not trash(sides.down) then go(sides.down) end
    if not trash(sides.up) then go(sides.up) end
    -- tonnel for player
    if PATHWAYS then
      if loc.y == halfw then
        robot.swingDown()
        while robot.detectUp() do robot.swingUp() end
      end
      if x == 1 or x == w then
        robot.swingDown()
      end
    else
      if loc.y == halfw then
        robot.swingDown()
      end
    end
    -- move
    if x<w then
      qforward()
      if PATHWAYS and tenth then robot.swingDown() end
    end
  end

  -- check front
  if not trash(sides.front) then go(sides.front) end

  -- next one
  if progress%2==1 then
    turnRight()
    qforward()
    turnRight()
  else
    turnLeft()
    qforward()
    turnLeft()
  end

  -- fuel checking
  check_fuel()
  -- trash checking
  if DROP_TRASH then
    check_trash()
  end
  -- loot checking
  if check_loot() then 
    turnAround()
    unload() 
    turnAround()
  end
end

function goBack()
  while loc.y ~= 0 do
    qforward()
  end
  if loc.d == 0 then turnLeft()
  elseif loc.d == 2 then turnRight() end
  while loc.x ~= 0 do
    qforward()
  end
  turnRight()
end

-- =================================== M A I N =================================== --
print("Totoro Recursive Miner 1.5H")
print("[INFO] Запуск. Длина: "..l..". Ширина: "..w..'. \n       Возврат: '..comeback)
if cloader ~= nil and USE_CLOADER then 
  cloader.setActive(true) 
  print("[INFO] Чанклодер активирован.")
end
print("[INFO] Выключите робота, чтобы прервать работу.")

-- statistic value
moves = 0
-- main cycle
robot.turnLeft()
if light_allowed then robot.setLightColor(color.working) end
for i=1, l do
  step(i)
end

-- ==================================== E N D ==================================== --
-- move to start position
if comeback == 'true' then goBack() end
if light_allowed then robot.setLightColor(color.idle) end
robot.turnRight()

if cloader ~= nil and USE_CLOADER then 
  cloader.setActive(false)
  print("[INFO] Чанклодер деактивирован.")
end
print("[STATISTIC] Движений: "..moves)
"#.as_bytes();

