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


math.randomseed(44)


local args = {}

local function add(mas,x,y,z,f) 
	mas[x] = mas[x] or {} 
	mas[x][y] = mas[x][y] or {} 
	mas[x][y][z] = f
end

local h = {}

local function testLife(xx,yy,zz,jjj)
	local life = 0
	for x = xx-1, xx+1 do
		for y = yy-1, yy+1, (x == xx and y == yy) and 2 or 1 do
			for z = zz-1, zz+1, (x == xx and y == yy) and 2 or 1 do
				if h[x] and h[x][y] and h[x][y][z] then life = life + 1 end
				--hologram.set(x,y,z,jjj) os.sleep(0.3) -- < >
			end
		end
	end
	return life
end


-- Set random pole

for _ = 1,2500 do
	add(h,math.random(8,40),math.random(8,24),math.random(8,40),true)
end

-- Set kub

-- for x = 16-1, 16+1 do
-- 	for y = 16-1, 16+1, (x == 16 or y == 16) and 2 or 1 do
-- 		for z = 16-1, 16+1, (x == 16 or y == 16 or z == 16) and 2 or 1 do
-- 			add(h,x,y,z,true)
-- 		end
-- 	end
-- end



hologram.clear()

for x,yz in pairs(h) do
	for y,zz in pairs(yz) do
		for z,fl in pairs(zz) do
			if fl then hologram.set(x,y,z,3) end
		end
	end
end

local rules = {}

for i,v in pairs(args) do
	rules[i] = args[i] and tonumber(args[i])
end

rules[1] = rules[1] or 5

if not rules[2] then
	rules[2] = 6
	rules[3] = 7
	rules[4] = 8
end

local function rulesValid(inPut)
	local flag
	for i = 2,4 do
		if not rules[i] then break end
		if inPut == rules[i] then flag = true end
	end
	return flag
end


while true do

	os.sleep(0.4)

	local newH, noValid = {}, {}
	for xx,vYZ in pairs(h) do
		for yy,vZ in pairs(vYZ) do
			for zz,out in pairs(vZ) do


				local test = testLife(xx,yy,zz,1)
				if rulesValid(test) then add(newH,xx,yy,zz,true) else hologram.set(xx,yy,zz,0) end
				
				for x = xx-1, xx+1 do
					for y = yy-1, yy+1, (x == xx and y == yy) and 2 or 1 do
						for z = zz-1, zz+1, (x == xx and y == yy) and 2 or 1 do
							if ( not (noValid[x] and noValid[x][y] and noValid[x][y][z]) ) and ( not (newH[x] and newH[x][y] and newH[x][y][z]) ) and (not(h[x] and h[x][y] and h[x][y][z])) and testLife(x,y,z,3) == rules[1] then 
								add(newH,x,y,z,true) hologram.set(x,y,z,3) 
							else 
								add(noValid,x,y,z,true) 
							end
						end
					end
				end


			end
		end
	end
	h = newH
end
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
            let mut fuel = Fuel::with(40960);
            while ex.step(ctx, &mut fuel) {
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
