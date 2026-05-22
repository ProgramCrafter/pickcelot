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
    palette = {[0]={0,0,0}},
    buffer = {},
    history = {},
    setPaletteColor = function(i, c)
        hologram.palette[i] = {(c-(c%65536))/65536,
                               ((c-(c%256))/256)%256, c%256}
    end,
    set = function(z, y, x, i)
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


local function pullSignal(t)
    os.sleep(t or 0.0)
end

local xor = function(u,v) return u ~ v end
local abs=math.abs
local cos=math.cos
local sin=math.sin
local char=function(c) return c > 32 and '.' or ' ' end
local time, this_time = os.time, 0
local center,max_x,max_y = 0, gpu.getResolution()
local ys,xs = max_y*4, max_x*2
local center_x=xs//2-2
local center_y = ys//2-2

local radius = ys//2-10
local x,y=0,0
--опишем более быстрое однобитное ксорирование
local reverse={} reverse[0]=1 reverse[1]=0
local chars,screen={},{}
local ch_y,ch_x = (max_y//1),(max_x//1)
local xr,yr=0,0
local mode='0'
actions={}
events = {key_up='keyUp'}
--chars init
for y=1,ch_y do
	chars[y]={}
end
--screen init
for y=1.0,ys do
	screen[y]={}
end
--перехват ивентов. надстройка над ОС
function computer.pullSignal(...)
    local e = {pullSignal(...)}
       if events[e[1]] then
           return actions[events[e[1]]](e)
       end
   return true --table.unpack(e) --true --table.unpack(e) 
end
-----------------------------------
actions.t=function()
    --tetminate
    mode='terminate'
    term.clear()
    screen=nil
    chars=nil
    computer.pullSignal = pullSignal
    evo=nil
    return os.exit()
end
actions['1']=function()
 mode='0'
 return true
end
actions['2']=function()
mode = '1'
return true
end
---------------------------------
actions.keyUp=function(e)
    local key=math.floor(e[3])
    if key > 128 then
        key = string.lower(ru_keys[key])
    else
        key=string.lower(string.char(key))
    end
    if actions[key] then
        return actions[key](e)
    end
    return true
end


local function cls_chr()
    for y = 1,ch_y do 
        for x = 1,ch_x do 
            chars[y][x] = 0x2800
        end
    end 
end

local function cls_scr()    
	for y=1.0,ys do
		for x=1.0,xs do
			screen[y][x]=0
		end
	end
end

--опишем биткарту шрифта брайля
local bits = {} 
	bits[1]={1,8,2,16,4,32,64,128}
	bits[0]={0,0,0,0,0,0,0,0}
	bits[-1]={-1,-8,-2,-16,-4,-32,-64,-128}
--попробуем описать трансформацию значений массива в шрифт брайля
local function toUnicode()
  local ch_x,ch_y,yy,xx=0,0,0,0
    for y in pairs(screen) do
        ch_y=y+3  yy=y-1
        ch_y=math.floor(ch_y/4)
        for x in pairs(screen[y]) do
          ch_x=x+1  xx=x-1
            ch_x=math.floor(ch_x/2)
            chars[ch_y][ch_x]=chars[ch_y][ch_x]+bits[screen[y][x]][1+(yy%4)*2+xx%2]
        end
    end
    return true
end

--отобразим содержимое экрана
local function showMustGoOne()
    for y in pairs(chars)do
        show=''
        
        for x in pairs(chars[y])do
            show = show..char(chars[y][x])
        end
        gpu.set(1,y+1,show)
    end
    return true
end

local function pseudo_draw(x1,y1,x,y)--вычисляет координаты точек линии
	if x < x1 then x_step = -1 else x_step = 1 end
	if y < y1 then y_step = -1 else y_step = 1 end

	if abs(x-x1) > abs(y-y1) then
		y_step=y_step*abs(y-y1)/abs(x-x1)
		y_plot=y1
		for x_plot = x1,x,x_step do
			screen[center_y+y_plot//1+1][center_x+x_plot//1+1]=reverse[screen[center_y+y_plot//1+1][center_x+x_plot//1+1]]
			y_plot=y_plot+y_step
		end
	else
		x_step=x_step*abs(x-x1)/abs(y-y1)
		x_plot=x1
		for y_plot = y1,y,y_step do
			screen[center_y+y_plot//1+1][center_x+x_plot//1+1]=reverse[screen[center_y+y_plot//1+1][center_x+x_plot//1+1]]
			x_plot=x_plot+x_step
		end
	end
end

cls_scr() cls_chr()
local x,y,a,angle,step,f,lines=0,0,0,0,17,2,0

function main()
	gpu.set(1,1,'Press key: (1) - mode 1, (2) - mode 2, (T) - for exit')
while math.huge do
	lines=512.0
	while lines <= 888 do
		this_time=time()/1000
		if mode == '1' then 
			xr=(this_time%(radius-radius/8))*cos(1.0/sin(this_time))--radius/2-4
			yr=(this_time%(radius-radius/8))*sin(1.0/cos(this_time))--radius/2-4
		else
			xr=0
			yr=0
		end
		angle=f*math.pi/lines
		a= 0
		for i = 1, lines do
			if mode == '1' then 
				xr=(this_time%(radius-radius/8))*cos(1.0/sin(this_time))
				yr=(this_time%(radius-radius/8))*sin(1.0/cos(this_time))
			else
				xr=0
				yr=0
			end
			x=radius*cos(a)
			y=radius*sin(a)
			--pseudo_draw(center_x/2,center_y/2,x,y)
			pseudo_draw(xr,yr,x,y)
			a=a+angle
		end

		toUnicode()
		showMustGoOne()
		cls_scr()
		cls_chr()
		os.sleep((8001-lines)/40000)
		os.sleep(0.3)
		lines=lines+step
	end
	if step <60 then step=step+3 else step=33 end
end
end
main()
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
