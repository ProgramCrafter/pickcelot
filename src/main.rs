use std::collections::HashMap;
use egui::*;


const PADDING: f32 = 16.0;
const FONT_SIZE: f32 = 17.0;

const _LUA_CODE_BASE: &[u8] = b"
local function gpu_set(...)
    return piccolo_component_invoke('$gpu', 'gpu.set', ...)
end
local function os_sleep(...)
    return piccolo_component_invoke('$os', 'os.sleep', ...)
end

local s = '0123456789abcdefghiklmnopqrstuvwxyz'
s = s .. '!treecollectionsmeanthebestofthebest$' .. s
local i = 0
while true do
    i = i + 1
    if i > 90 then i = 1 end
    
    gpu_set(i + 1, 1, string.sub(s, i, i), false)
    gpu_set(i, 1, ' ', false)
    
    if i % 12 == 0 then
        os_sleep(0.8)
    elseif i % 3 == 0 then
        piccolo_yield()
    end
end
";

const LUA_CODE: &[u8] = "
-- Автор: qwertyMAN
-- Версия: 0.1 бета

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
    getResolution = function() return 90, 35 end,
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
        local w, h = gpu.getResolution()
        local s = string.rep(' ', w)
        for i = 1, h, 1 do
            gpu.set(1, i, s, false)
        end
    end
}
local os = {
    sleep = function(t)
        return piccolo_component_invoke('$os', 'os.sleep', t)
    end,
    time = function() return 1.0 end
}

local display			= {gpu.getResolution()}
local players			= {}		-- свойства игроков
local players_vector	= {}		-- направления игроков
local area				= {}		-- игровое поле
local turn				= {}		-- таблица поворота
local target			= {}		-- координаты цели
local border			= {0,0}		-- неиспользуемый отступ от края экрана
local t					= 1/8		-- скорость игры
local exit_game			= false		-- для выхода из игры
local size				= {math.modf((display[1]-border[1])/2), math.modf(display[2]-border[2])} -- размер игрового поля
math.randomseed(os.time())

turn[1]={0,-1}
turn[2]={0,1}
turn[3]={-1,0}
turn[4]={1,0}

local command={}					-- управление:
command[200]=function(nick)			-- вверх
	if players_vector[nick] ~= 2 then
		players_vector[nick] = 1
	end
end 
command[208]=function(nick)			-- вниз
	if players_vector[nick] ~= 1 then
		players_vector[nick] = 2
	end
end
command[203]=function(nick)			-- влево
	if players_vector[nick] ~= 4 then
		players_vector[nick] = 3
	end
end
command[205]=function(nick)			-- вправо
	if players_vector[nick] ~= 3 then
		players_vector[nick] = 4
	end
end

-- генерация поля
for i=1, size[1] do
	area[i]={}
	for j=1, size[2] do
		area[i][j]=false
	end
end

local function conv_cord(sx,sy)
	return sx*2-1+border[1], sy+border[2]
end

local function gen_target()
	while true do
		local x,y = math.random(1,size[1]), math.random(1,size[2])
		if not area[x][y] then
			target = {x,y}
			gpu.setBackground(0x0000ff)
			local rezerv = {conv_cord(x,y)}
			gpu.set(rezerv[1], rezerv[2], '  ')
			gpu.setBackground(0x000000)
			break
		end
	end
end

local function keyboard(_,_,_,key,nick)
	local swich = true
	for i=1, #players do
		if nick==players[i].name then
			swich = false
		end
	end
	if swich and (key==200 or key == 203 or key == 205 or key == 208) then
		-- если игрока нет в списке
		players[#players+1]={name=nick,number=5,cord={5,5}}
		area[players[#players].cord[1]][players[#players].cord[2]]=players[#players].number
	end
	if key == 16 then -- выход
		exit_game = true
	elseif command[key] then
		command[key](nick)
	end
end

local function update()
	-- проверка есть ли препятствие
	for i=#players, 1, -1  do
		local cord = turn[players_vector[players[i].name]]
		local cord_2 = {players[i].cord[1]+cord[1],players[i].cord[2]+cord[2]}
		gpu.setBackground(0xffffff)
		local rezerv = {conv_cord(players[i].cord[1], players[i].cord[2])}
		gpu.set(rezerv[1], rezerv[2], '  ')
		gpu.setBackground(0x000000)
		if cord_2[1]>size[1] then
			cord_2[1] = 1
		elseif cord_2[1] < 1 then
			cord_2[1] = size[1]
		elseif cord_2[2]>size[2] then
			cord_2[2] = 1
		elseif cord_2[2] < 1 then
			cord_2[2] = size[2]
		end
		if not area[cord_2[1]][cord_2[2]] then
			players[i].cord[1]=cord_2[1]
			players[i].cord[2]=cord_2[2]
			area[players[i].cord[1]][players[i].cord[2]]=players[i].number
			gpu.setBackground(0x00ff00)
			gpu.setForeground(0x000000)
			local rezerv = {conv_cord(players[i].cord[1], players[i].cord[2])}
			gpu.set(rezerv[1], rezerv[2], string.sub(players[i].name,1,2))
			if target[1]==players[i].cord[1] and target[2]==players[i].cord[2] then
				players[i].number = players[i].number+1
				gen_target()
			end
		else
			table.remove(players,i)
		end
		gpu.setBackground(0x000000)
		gpu.setForeground(0xffffff)
	end
	
	-- обновление и добавление ячеек
	for i=1, size[1] do
		for j=1, size[2] do
			if area[i][j] then
				if area[i][j]>0 then
					area[i][j]=area[i][j]-1
				else
					area[i][j]=false
					gpu.setBackground(0x000000)
					local rezerv = {conv_cord(i,j)}
					gpu.set(rezerv[1], rezerv[2], '  ')
				end
			end
			
		end
	end
end

-- очищаем экран
gpu.setBackground(0x000000)
gpu.setForeground(0xffffff)
term.clear()
event.listen('key_down', keyboard)

gen_target()

-- тело игры
while true do
	os.sleep(t)
	if exit_game then
		term.clear()
		print('Exit game')
		os.sleep(2)
		term.clear()
		return
	end
	update()
end

term.clear()
".as_bytes();


mod screen {
    use crate::*;
    
    type ScreenChar = (char, Color32, Color32);
    const BLANK: ScreenChar = ('i', Color32::WHITE, Color32::DARK_BLUE);
    
    pub struct Screen<const W: usize, const H: usize> {
        pub chars: Box<[[ScreenChar; W]; H]>,
    }
    
    impl<const W: usize, const H: usize> Default for Screen<W, H> {
        fn default() -> Self { Self::new() }
    }
    
    impl<const W: usize, const H: usize> Screen<W, H> {
        fn new() -> Self {
            Self {
                chars: Box::new([[BLANK; W]; H]),
            }
        }
        fn draw_callback(&self, mut painter: impl FnMut(f32, f32, ScreenChar)) {
            for y in 0..H {
                for (x, schar) in self.chars[y].iter().enumerate().rev() {
                    painter(x as f32, y as f32, *schar);
                }
            }
        }
    }
    
    impl<'a, const W: usize, const H: usize> Widget for &'a Screen<W, H> {
        fn ui(self, ui: &mut Ui) -> Response {
            // 1. Space requirements.
            let font_id = FontId::monospace(FONT_SIZE);
            let (cell_w, cell_h) = ui.ctx().fonts_mut(|f| {
                (f.glyph_width(&font_id, '~'), f.row_height(&font_id))
            });
            assert_ne!(cell_w, 0.0, "font missing characters like '~'");
            let grid = vec2(cell_w * (W as f32), cell_h * (H as f32));
            let cell = vec2(cell_w, cell_h);
            let sense = Sense::empty();
            
            // 2. Space.
            let (response, painter) = ui.allocate_painter(grid, sense);
            let rect = response.rect;
            
            // 3. Drawing.
            self.draw_callback(|x, y, (ch, fg, bg)| {
                let pos = rect.min + vec2(x * cell_w, y * cell_h);
                let cell_rect = Rect::from_min_size(pos, cell);

                painter.rect_filled(cell_rect, 0.0, bg);
                painter.text(pos, Align2::LEFT_TOP, ch, font_id.clone(), fg);
            });
            
            response
        }
    }
}


type InvocationVariadic = Vec<piccolo::Constant<Box<[u8]>>>;
struct ComponentInvoke {
    target:   (String, String),
    variadic: InvocationVariadic,
}
impl ComponentInvoke {
    #[must_use]
    fn make<'gc, 'a>(
        ctx: piccolo::Context<'gc>,
        stack: &mut piccolo::Stack<'gc, 'a>,
    ) -> Result<(), piccolo::TypeError> {
        use piccolo::*;
        use std::string::String;
        type Manyval<'gc> = Variadic<Vec<Value<'gc>>>;
        
        let args: Result<(String,String,Manyval), _> = stack.consume(ctx);
        let (address, method, variadic_ref) = args?;
        let variadic = variadic_ref.0.into_iter().map(|val| {
            let c = val.to_constant().unwrap_or(Constant::Nil);
            c.map_string(|s| s.as_bytes().into())
        }).collect();
        
        let ud = UserData::new_static(&ctx, ComponentInvoke {
            target: (address, method),
            variadic,
        });
        stack.replace(ctx, ud);
        
        Ok(())
    }
}



struct Peripherals {
    screen: screen::Screen<90, 35>,
    gpu:    (Color32, Color32),
    sleep:  f64,
    timer:  f64,
}

struct App {
    config: piccolo::Lua,
    engine: piccolo::StashedExecutor,
    
    peripheral: Peripherals,
    components: HashMap<(String, String),
        fn(&mut Peripherals, &InvocationVariadic) -> bool>,
}

impl Default for App {
    fn default() -> Self {
        // 1. Initialize Lua part with callbacks.
        // no: IO
        // ok: base, coroutine, math, string, table
        let mut config = piccolo::Lua::core();
        
        let engine = config.enter(|ctx| {
            use piccolo::*;
            
            let piccolo_yield = Callback::from_fn(&ctx, |_, mut exec, _| {
                exec.fuel().interrupt();
                Ok(CallbackReturn::Return)
            });
            _ = ctx.set_global("piccolo_yield", piccolo_yield).unwrap();
            
            let ci = Callback::from_fn(&ctx, |ctx, _, mut stack| {
                ComponentInvoke::make(ctx, &mut stack)?;
                Ok(CallbackReturn::Yield {to_thread: None, then: None})
            });
            _ = ctx.set_global("piccolo_component_invoke", ci).unwrap();
            
            let cl = Closure::load(ctx, None, LUA_CODE).unwrap();
            let ex = Executor::start(ctx, cl.into(), ());
            ctx.stash(ex)
        });
        
        // 2. Initialize peripherals.
        let peripheral = Peripherals {
            screen: Default::default(),
            gpu:    (Color32::WHITE, Color32::DARK_BLUE),
            sleep:  f64::NEG_INFINITY,
            timer:  f64::NAN,
        };
        
        // 3. Callbacks for peripherals.
        
        let mut components = HashMap::new();
        components.insert(("$gpu".to_owned(), "gpu.set".to_owned()),
        |p: &mut Peripherals, args: &InvocationVariadic| -> bool {
            use piccolo::Constant::*;
            
            let (cx, cy, cs, cd) = match &args[..] {
                [cx, cy, cs, cd] => (cx, cy, cs, cd),
                _                => return false,
            };
            
            let x = match cx {
                Integer(n) => *n,
                Number(n)  => n.trunc() as i64,
                _          => return false,
            };
            let y = match cy {
                Integer(n) => *n,
                Number(n)  => n.trunc() as i64,
                _          => return false,
            };
            let s = match cs {
                String(s)  => s,
                _          => return false,
            };
            let d = match cd {
                Boolean(d) => *d,
                _          => return false,
            };
            
            let Ok(x) = usize::try_from(x - 1) else {return false};
            let Ok(y) = usize::try_from(y - 1) else {return false};
            let Ok(s) = str::from_utf8(&s) else {return false};
            
            let gpu_config = p.gpu;
            
            for (i, c) in s.chars().enumerate() {
                let (char_x, char_y) = match d {
                    false => (x.saturating_add(i), y),
                    true  => (x, y.saturating_add(i)),
                };
                if let Some(row) = p.screen.chars.get_mut(char_y) {
                    if let Some(col) = row.get_mut(char_x) {
                        *col = (c, gpu_config.0, gpu_config.1);
                    }
                }
            }
            
            true
        } as _);
        components.insert(("$gpu".to_owned(), "gpu.setForeground".to_owned()),
        |p: &mut Peripherals, args: &InvocationVariadic| -> bool {
            let Some(t) = (match &args[..] {
                [ct] => ct.to_integer(),
                _    => return false,
            }) else {return false};
            
            let (b, rg) = (t % 256, t / 256);
            let (g, r) = (rg % 256, rg / 256);
            
            p.gpu.0 = Color32::from_rgb(r as u8, g as u8, b as u8);
            
            true
        } as _);
        components.insert(("$gpu".to_owned(), "gpu.setBackground".to_owned()),
        |p: &mut Peripherals, args: &InvocationVariadic| -> bool {
            let Some(t) = (match &args[..] {
                [ct] => ct.to_integer(),
                _    => return false,
            }) else {return false};
            
            let (b, rg) = (t % 256, t / 256);
            let (g, r) = (rg % 256, rg / 256);
            
            p.gpu.1 = Color32::from_rgb(r as u8, g as u8, b as u8);
            
            true
        } as _);
        components.insert(("$os".to_owned(), "os.sleep".to_owned()),
        |p: &mut Peripherals, args: &InvocationVariadic| -> bool {
            let Some(t) = (match &args[..] {
                [ct] => ct.to_number(),
                _    => return false,
            }) else {return false};
            
            p.sleep = p.timer + t;
            
            true
        } as _);
        
        
        Self {
            config,
            engine,
            peripheral,
            components,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ectx: &Context, _frame: &mut eframe::Frame) {
        self.peripheral.timer = ectx.input(|i| i.time);
        
        self.config.enter(|ctx| {
            use piccolo::*;
            
            let sleep_t = self.peripheral.sleep - self.peripheral.timer;
            if sleep_t > 0.0 {
                ectx.request_repaint_after_secs(sleep_t as f32);
                return;
            }
            self.peripheral.sleep = f64::NEG_INFINITY;
            
            let ex = ctx.fetch(&self.engine);
            if ex.step(ctx, &mut Fuel::with(16384)) {
                let request = ex
                    .take_result::<UserData>(ctx)
                    .map(|ok_mode| ok_mode.map(
                        |ud| ud.downcast_static::<ComponentInvoke>()
                    ));
                
                let iv: &ComponentInvoke = match request {
                    Err(bad_exec) => panic!("bad_exec={bad_exec}"),
                    Ok(Err(thread)) => panic!("thread_err={thread:?}"),
                    Ok(Ok(Err(_bad_ud))) => panic!("not ComponentInvoke"),
                    Ok(Ok(Ok(iv))) => iv,
                };
                
                if let Some(cb) = self.components.get(&iv.target)
                    && (cb)(&mut self.peripheral, &iv.variadic) {
                    ex.resume(ctx, ()).expect("yielded, cb complete");
                } else {
                    let s = String::from_slice(&ctx, "bad invocation");
                    let e = Error::from_value(s.into());
                    ex.resume_err(&ctx, e).expect("yielded, to resume");
                }
            }
            
            if ex.mode() != ExecutorMode::Stopped {
                ectx.request_repaint_after_secs(0.04);
            }
        });
        
        CentralPanel::default()
            .frame(Frame::new().inner_margin(PADDING))
            .show(ectx, |ui| {
                ui.add(&self.peripheral.screen);
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1032.0, 832.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Screen",
        options,
        Box::new(|_| Ok(Box::new(App::default()))),
    )
}
