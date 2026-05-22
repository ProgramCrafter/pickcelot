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
local os = {
    sleep = function(t)
        return piccolo_component_invoke('$os', 'os.sleep', t)
    end,
    time = function() return 1.0 end
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


local c = 20
--моделируем Снеговика
local tSnowman = {10,10,11,11,11,12,12,12,12,11,11,11,10,9,8,7,6,7,8,8,9,9,9,9,9,8,8,7,6,7,6,6,6,6,6,6}
--таблица падающих снежинок(Взаимствована у Doob, надеюсь он не против)
local tSnow = {}
for sn = 1, 32 do
  tSnow[sn] = {}
end
--палитра цветов
hologram.setPaletteColor(1, 0xFFFFFF)--белый
hologram.setPaletteColor(2, 0x8B0000)--бардовый
hologram.setPaletteColor(3, 0xFFA500)--оранжевый

--цикл рисования круга так же взаимствован у Doob
local function circle(x0, y, z0, R, i)
  local x = R
  local z = 0
  local err = -R
  while z <= x do
    hologram.set(x + x0, y, z+z0, i)
    hologram.set(z + x0, y, x + z0, i)
    hologram.set(-x + x0, y, z + z0, i)
    hologram.set(-z + x0, y, x + z0, i)
    hologram.set(-x + x0, y, -z + z0, i)
    hologram.set(-z + x0, y, -x + z0, i)
    hologram.set(x + x0, y, -z + z0, i)
    hologram.set(z + x0, y, -x + z0, i)
    z = z+1
    if err <= 0 then
      err = err + (2 * z + 1)
    else
      x = x - 1
      err = err + (2 * (z - x) + 1)
    end
  end
end
--рисуем пуговицы они же глаза
local function stud(x, y0, z0, i)
  for y = 0, 1 do
    for z = 0, 1 do
      hologram.set(x, y0-y, z0-z, i)
    end
  end
end
-- рисуем снеговика
local function snowman()
  for i = 1, 30 do
    circle(c,i,c, tSnowman[i], 1)--отрисовываем основу снеговика
  end
  for i = 30, #tSnowman do
    circle(c,i,c, tSnowman[i], 2)--рисуем шапку
  end
  --рисуем глаза
  stud(29, 23, 23, 3)
  stud(29, 23, 17, 3)
  --рисуем морковку
  for x = 1, 6  do
    for y = 1, 2 do
      for z = 1, 2 do
        hologram.set(27+x, 18+z, 18+y, 3)
      end
    end
  end
  -- правая рука
  for x = 1, 7 do
    y = 3
    hologram.set(20+y, 10+x, 10-x, 3)
  end
  -- левая рука
  for x = 1, 7 do
    y = 1
    for y = 1, 3 do
      y = y
    end
    hologram.set(20-y, 10+x, 31+x, 3)
  end
  --рисуем пуговицы
  stud(30, 14, 20, 2)
  stud(32, 10, 20, 2)
  stud(32, 6, 20, 2)

end

local function gen_snow() -- генерируем таблицу для источника снежинок(из кода Doob)
  local tbl = {}
  local x, y
  for g = 1, 10 do
    x, y = math.random(1, 46), math.random(1, 46)
    if tbl[x] == nil then
      tbl[x] = {}
    end
    tbl[x][y] = 1
  end
  return tbl
end

local function falling_snow() -- слвигаем таблицы со снежинками вниз(Doob)
  tmp = {}
  for s = 1, 32 do
    tmp[s] = {}
    tmp[s] = tSnow[s+1]
  end
  tSnow = tmp
  tSnow[32] = gen_snow()
end
 
local function animation(de) -- задаем указанное значение для снежинок(Doob)
  for x = 1, 46 do
    for z = 1, 46 do
      for y = 1, 32 do
        if tSnow[y][z] ~= nil then
          if tSnow[y][z][x] == 1 then
            hologram.set(x, y, z, de)
          end
        end
      end
    end
  end
end

hologram.clear()

while 1 do
  snowman()
  falling_snow()
  animation(1)
  os.sleep(2)
  animation(0)
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
            let mut fuel = Fuel::with(4096);
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
