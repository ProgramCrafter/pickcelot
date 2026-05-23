use std::collections::{BTreeMap, HashMap};
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
    getResolution = function()
        return piccolo_component_invoke('$gpu', 'gpu.getResolution')
    end,
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


-- GPU Waveform Demo
-- Tests gpu.set, gpu.setBackground, gpu.setForeground with different parameters
-- Y axis runs down, screen size 90x35 characters

-- Pre-build fixed strings to avoid string.rep (only string.sub allowed)
local SPACE35 = '                                   '  -- 35 spaces
local PIPE35  = '|||||||||||||||||||||||||||||||||||'  -- 35 pipes

local phase = 0
local center = 21.5
local amplitude = 11
local frequency = 3  -- waves across the width

local w, h = gpu.getResolution()

gpu.setBackground(0)

-- Main animation loop
while true do
    -- Clear the whole screen column by column using vertical space strings
    gpu.setForeground(0xFFFFFF)
    --[[
    for y = 1, h do
        gpu.set(1, y, string.rep(' ', w))  -- vertical placement, clears column
    end
    --]]

    -- Draw title (horizontal placement)
    gpu.set(25, 1, 'GPU Waveform Demo', true)
    gpu.set(15, 3, 'place_vertically=true for bars, nil here')

    -- Draw waveform bars with vertical text + point markers
    for x = 1, w do
        local wave = center + amplitude * math.sin((x / w) * 2 * math.pi * frequency + phase)
        local y_wave = math.floor(wave + 0.5)
        if y_wave < 1 then y_wave = 1
        elseif y_wave > h then y_wave = h end

        -- Dynamic colour based on bar height (blue -> red)
        local r = math.floor(255 * (y_wave / h) ^ 0.5)
        local g = math.floor(255 * math.max(0, y_wave - 29) / 12)
        local b = math.floor(255 * (1 - y_wave / h) ^ 0.5)
        local color = r * 65536 + g * 256 + b

        gpu.setForeground(color)

        -- Vertical bar using place_vertically = true
        -- Height = y_wave, starts at row 1, goes downwards
        local bar = string.sub(PIPE35, 1, y_wave)
        local ter = string.sub(SPACE35, 1, amplitude + center + 0.5 - y_wave)
        gpu.set(x, 1, bar .. ter, true)

        -- Overlay the actual wave point as a solid block (horizontal placement)
        gpu.setForeground(0x24 * 0x010101)
        gpu.set(x, y_wave, '█', false)
    end

    -- Add a moving horizontal label at the bottom to test horizontal placement again
    gpu.setForeground(0xFFFF00)
    local label = ' Phase: ' .. tostring(phase) .. '   '
    local labelX = math.floor(45 + 20 * math.sin(phase * 0.7))
    gpu.set(labelX, 34, label, false)

    os.sleep(0.07)
    phase = phase + 0.0625
end
".as_bytes();


mod screen {
    use crate::*;
    
    type ScreenChar = (char, Color32, Color32);
    const BLANK: ScreenChar = ('i', Color32::WHITE, Color32::DARK_BLUE);
    
    pub struct Screen<const W: usize, const H: usize> {
        pub chars: Box<[[ScreenChar; W]; H]>,
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
        
        pub fn put(&mut self, c: char, colors: (Color32, Color32),
                   location: &mut (i64, i64), carriage_vertical: bool) {
            let (ix, iy) = *location;
            if let Ok(y) = usize::try_from(iy) && y < H
            && let Ok(x) = usize::try_from(ix) && x < W {
                self.chars[y][x] = (c, colors.0, colors.1);
            }
            
            if carriage_vertical {
                location.1 = iy.saturating_add(1);
            } else {
                let char_width = if c.is_control() { 0 } else { 1 };
                location.0 = ix.saturating_add(char_width);
            }
        }
    }
    
    impl<const W: usize, const H: usize> Default for Screen<W, H> {
        fn default() -> Self { Self::new() }
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


type Constant = piccolo::Constant<Box<[u8]>>;
type LuaList = Vec<Constant>;
type InvResult = Result<LuaList, anyhow::Error>;
struct ComponentInvoke {
    target:   (String, String),
    variadic: LuaList,
}
impl ComponentInvoke {
    #[must_use]
    fn make<'gc, 'a>(
        ctx: piccolo::Context<'gc>,
        stack: &mut piccolo::Stack<'gc, 'a>,
    ) -> Result<(), piccolo::TypeError> {
        use piccolo::*;
        use std::string::String;
        type Manyval<'gc> = piccolo::Variadic<Vec<piccolo::Value<'gc>>>;
        
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
    
    #[must_use]
    fn stash_vec<'gc>(
        ctx: piccolo::Context<'gc>,
        results: LuaList,
    ) -> piccolo::Variadic<Vec<piccolo::Value<'gc>>> {
        use piccolo::*;
        
        Variadic(results
            .into_iter()
            .map(|c| match c {
                Constant::String(s) => Value::String(
                    String::from_buffer(&ctx, s)
                ),
                Constant::Boolean(b) => Value::Boolean(b),
                Constant::Integer(i) => Value::Integer(i),
                Constant::Number(n) => Value::Number(n),
                Constant::Nil => Value::Nil,
            })
            .collect())
    }
}

fn ld_int(v: &Constant) -> Result<i64, anyhow::Error> {
    match v {
        Constant::Integer(n) => Ok(*n),
        Constant::Number(n)  => Ok(n.trunc() as i64),
        _ => Err(anyhow::anyhow!("expected integer/number value")),
    }
}
fn ld_bool(v: &Constant) -> Result<bool, anyhow::Error> {
    match v {
        Constant::Boolean(b) => Ok(*b),
        _ => Err(anyhow::anyhow!("expected boolean value")),
    }
}
fn ld_str(v: &Constant) -> Result<&str, anyhow::Error> {
    match v {
        Constant::String(s) => Ok(str::from_utf8(s)?),
        _ => Err(anyhow::anyhow!("expected string value")),
    }
}
fn asize() -> anyhow::Error {
    anyhow::anyhow!("wrong number of arguments passed")
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
        fn(&mut Peripherals, &LuaList) -> InvResult>,
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
        components.insert(
            ("$gpu".to_owned(), "gpu.set".to_owned()),
            |p: &mut Peripherals, args: &LuaList| -> InvResult {
                let [cx, cy, cs, cd] = args.as_array().ok_or_else(asize)?;
                let mut pos = (ld_int(cx)?.saturating_sub(1),
                               ld_int(cy)?.saturating_sub(1));
                let d = ld_bool(cd)?;
                
                if let Ok(one_char) = ld_int(cs) {
                    let c = char::try_from(u32::try_from(one_char)?)?;
                    p.screen.put(c, p.gpu, &mut pos, d);
                } else {
                    for c in ld_str(cs)?.chars() {
                        p.screen.put(c, p.gpu, &mut pos, d);
                    }
                }
                Ok(vec![])
            } as _
        );
        
        components.insert(
            ("$gpu".to_owned(), "gpu.getResolution".to_owned()),
            |p: &mut Peripherals, args: &LuaList| -> InvResult {
                let [] = args.as_array().ok_or_else(|| anyhow::anyhow!("wrong number of arguments passed: 0 expected"))?;
                
                let _screen: &screen::Screen<90, 35> = &p.screen;
                Ok(vec![
                    piccolo::Constant::Integer(90),
                    piccolo::Constant::Integer(35),
                ])
            } as _
        );
        
        components.insert(
            ("$gpu".to_owned(), "gpu.setForeground".to_owned()),
            |p: &mut Peripherals, args: &LuaList| -> InvResult {
                let [color] = args.as_array().ok_or_else(|| anyhow::anyhow!("wrong number of arguments passed: 1 expected"))?;
                let color = color.to_integer().ok_or_else(|| anyhow::anyhow!("int conversion failed"))?;
                let r = u8::try_from(color / 65536)?;
                let g = u8::try_from((color % 65536) / 256)?;
                let b = u8::try_from(color % 256)?;
                p.gpu.0 = Color32::from_rgb(r, g, b);
                Ok(vec![])
            } as _
        );
        
        components.insert(
            ("$gpu".to_owned(), "gpu.setBackground".to_owned()),
            |p: &mut Peripherals, args: &LuaList| -> InvResult {
                let [color] = args.as_array().ok_or_else(|| anyhow::anyhow!("wrong number of arguments passed: 1 expected"))?;
                let color = color.to_integer().ok_or_else(|| anyhow::anyhow!("int conversion failed"))?;
                let r = u8::try_from(color / 65536)?;
                let g = u8::try_from((color % 65536) / 256)?;
                let b = u8::try_from(color % 256)?;
                p.gpu.1 = Color32::from_rgb(r, g, b);
                Ok(vec![])
            } as _
        );
        
        components.insert(
            ("$os".to_owned(), "os.sleep".to_owned()),
            |p: &mut Peripherals, args: &LuaList| -> InvResult {
                let [time] = args.as_array().ok_or_else(|| anyhow::anyhow!("wrong number of arguments passed: 1 expected"))?;
                p.sleep = p.timer + time.to_number().ok_or_else(|| anyhow::anyhow!("number conversion failed"))?;
                Ok(vec![])
            } as _
        );
        
        
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
        
        let mut invocations = 0;
        
        self.config.enter(|ctx| {
            use piccolo::*;
            
            let sleep_t = self.peripheral.sleep - self.peripheral.timer;
            if sleep_t > 0.0 {
                ectx.request_repaint_after_secs(sleep_t as f32);
                return;
            }
            self.peripheral.sleep = f64::NEG_INFINITY;
            
            let ex = ctx.fetch(&self.engine);
            let mut fuel = Fuel::with(4096 * 30);
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
                
                invocations += 1;
                
                if let Some(cb) = self.components.get(&iv.target) {
                    match (cb)(&mut self.peripheral, &iv.variadic) {
                        Ok(v) => ex.resume(ctx, ComponentInvoke::stash_vec(ctx, v)),
                        Err(e) => ex.resume_err(&ctx, e.into()),
                    }.expect("resuming should work because callback can return");
                    
                    if self.peripheral.sleep > self.peripheral.timer {
                        break;
                    }
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
                ui.add_space(PADDING);
                
                if invocations != 0 {
                    ui.label(format!("component.invoke(): {invocations}"));
                } else {
                    ui.label("component.invoke(): ___");
                }
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
