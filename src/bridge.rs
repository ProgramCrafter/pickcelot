use piccolo::*;
use std::string::String;

type NonConstant<'gc> = Value<'gc>;
pub type BoxConstant = Constant<Box<[u8]>>;

pub type Manyval<'gc> = Variadic<Vec<NonConstant<'gc>>>;
pub type LuaList = Vec<BoxConstant>;

type MkResult = Result<(), TypeError>;


pub struct ComponentInvoke {
    pub target:   (String, String),
    pub variadic: LuaList,
}

impl ComponentInvoke {
    #[must_use = "verify result or risk empty Lua stack"]
    pub fn make<'gc>(ctx: Context<'gc>, stack: &mut Stack<'gc,'_>) -> MkResult {
        let args: Result<(String, String, Manyval), _> = stack.consume(ctx);
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
    pub fn stash_vec(ctx: Context<'_>, results: LuaList) -> Manyval<'_> {
        Variadic(results
            .into_iter()
            .map(|c| match c {
                Constant::String(s) => Value::String(
                    piccolo::String::from_buffer(&ctx, s)
                ),
                Constant::Boolean(b) => Value::Boolean(b),
                Constant::Integer(i) => Value::Integer(i),
                Constant::Number(n) => Value::Number(n),
                Constant::Nil => Value::Nil,
            })
            .collect())
    }
}

pub fn ld_int(v: &BoxConstant) -> Result<i64, anyhow::Error> {
    match v {
        Constant::Integer(n) => Ok(*n),
        Constant::Number(n)  => Ok(n.trunc() as i64),
        _ => Err(anyhow::anyhow!("expected integer/number value")),
    }
}
pub fn ld_bool(v: &BoxConstant) -> Result<bool, anyhow::Error> {
    match v {
        Constant::Boolean(b) => Ok(*b),
        _ => Err(anyhow::anyhow!("expected boolean value")),
    }
}
pub fn ld_str(v: &BoxConstant) -> Result<&str, anyhow::Error> {
    match v {
        Constant::String(s) => Ok(str::from_utf8(s)?),
        _ => Err(anyhow::anyhow!("expected string value")),
    }
}

#[macro_export]
macro_rules! read_or_bail {
    ([$($binding:ident),*] = $stack:expr) => {
        let Some([$($binding),*]) = $stack.as_array() else {
            let mut n = 1;
            n -= 1;
            
            $(
                #[expect(unused_variables)]
                let $binding: ();
                n += 1;
            )*
            
            return ::std::result::Result::Err(
                ::anyhow::anyhow!(
                    "wrong number of arguments passed: {n} expected"
                )
            )
        };
    }
}

