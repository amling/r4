#[cfg(test)]
mod tests;

use record::F64HashDishonorProxy;
use record::JsonPrimitive;
use record::MRecord;
use record::Record;
use record::RecordNode;
use record::RecordTrait;
use rlua::Lua;
use rlua::MetaMethod;
use rlua::ToLua;
use rlua::UserData;
use rlua::UserDataMethods;
use rlua::Value;
use std::sync::Arc;
use super::ExecutorBe;
use super::ExecutorRegistrant;
use validates::ValidationResult;

#[derive(Clone)]
struct MRecordHolder(MRecord);

impl UserData for MRecordHolder {
    fn add_methods<'lua, M: UserDataMethods<'lua, MRecordHolder>>(m: &mut M) {
        m.add_meta_method_mut(MetaMethod::Index, |lua, r, k: Value| {
            return r.0.visit_converted(
                |rn| {
                    let ret;
                    match rn {
                        RecordNode::Primitive(_p) => {
                            panic!();
                        }
                        RecordNode::Array(arr) => {
                            let k = lua.coerce_integer(k).unwrap() as usize;
                            ret = arr[k - 1].clone();
                        }
                        RecordNode::Hash(hash) => {
                            let k: Arc<str> = Arc::from(lua.coerce_string(k).unwrap().to_str().unwrap());
                            ret = hash[&k].clone();
                        }
                    }
                    return to_lua(lua, ret);
                }
            );
        });
        m.add_meta_method_mut(MetaMethod::NewIndex, |lua, r, (k, v): (Value, Value)| {
            let v: MRecord = from_lua(lua, v);
            return r.0.visit_converted(
                |rn| {
                    match rn {
                        RecordNode::Primitive(_p) => {
                            panic!();
                        }
                        RecordNode::Array(arr) => {
                            let k = lua.coerce_integer(k).unwrap() as usize;
                            arr[k - 1] = v;
                        }
                        RecordNode::Hash(hash) => {
                            let k: Arc<str> = Arc::from(lua.coerce_string(k).unwrap().to_str().unwrap());
                            hash.insert(k, v);
                        }
                    }
                    return Result::Ok(());
                }
            );
        });
    }
}

fn to_lua(lua: &Lua, r: MRecord) -> Result<Value, rlua::Error> {
    if let Some(p) = r.maybe_primitive() {
        return match p {
            JsonPrimitive::Null() => Result::Ok(Value::Nil),
            JsonPrimitive::Bool(b) => b.to_lua(lua),
            JsonPrimitive::NumberI64(n) => n.to_lua(lua),
            JsonPrimitive::NumberF64(F64HashDishonorProxy(f)) => f.to_lua(lua),
            JsonPrimitive::String(s) => s.to_lua(lua),
        };
    }
    return MRecordHolder(r.clone()).to_lua(lua);
}

fn from_lua(lua: &Lua, v: Value) -> MRecord {
    match v {
        Value::Nil => {
            return MRecord::null();
        }
        Value::Boolean(b) => {
            return MRecord::from(b);
        }
        Value::Integer(n) => {
            return MRecord::from(n);
        }
        Value::Number(n) => {
            // Oh boy, no integers in lua?  Coerce what we can back to i64.
            let ni = n as i64;
            let nf = n as f64;
            if (ni as f64) == nf {
                return MRecord::from(ni);
            }
            return MRecord::from(nf);
        }
        Value::String(s) => {
            return MRecord::from(s.to_str().unwrap());
        }
        Value::Table(t) => {
            return MRecord::from_hash(t.pairs::<Value, Value>().map(|p| {
                let (k, v) = p.unwrap();
                let k = Arc::from(lua.coerce_string(k).unwrap().to_str().unwrap());
                return (k, from_lua(lua, v));
            }).collect());
        }
        Value::UserData(ud) => {
            if let Result::Ok(r) = ud.borrow::<MRecordHolder>() {
                return r.0.clone();
            }
            panic!();
        }
        _ => {
            panic!();
        }
    }
}

pub(crate) type Impl = ExecutorRegistrant<ImplBe>;
pub(crate) struct ImplBe();

impl ExecutorBe for ImplBe {
    type Code = String;

    fn names() -> Vec<&'static str> {
        return vec!["lua"];
    }

    fn help_msg() -> &'static str {
        return "evaluate code using rlua";
    }

    fn parse(code: &str) -> ValidationResult<String> {
        return Result::Ok(code.to_string());
    }

    fn stream(code: &String, ret: bool) -> Box<dyn FnMut(Record) -> Record> {
        let lua = Lua::new();

        // Our library of functions to help manage API "issues".
        lua.globals().set("arr", lua.create_function(|lua, t: rlua::Table| {
            return MRecordHolder(MRecord::from_vec(t.sequence_values().map(|v| from_lua(lua, v.unwrap())).collect())).to_lua(lua);
        }).unwrap()).unwrap();

        // Your "main" function.  We hold a RegistryKey since basically
        // anything else is lifetime tied to lua object and we therefore simply
        // can't keep them.
        let f = lua.create_registry_value(lua.load(&code, None).unwrap()).unwrap();

        return Box::new(move |r| {
            lua.globals().set("r", MRecordHolder(MRecord::wrap(r))).unwrap();

            let f: rlua::Function = lua.registry_value(&f).unwrap();

            let r: Value;
            if ret {
                r = f.call(()).unwrap();
            }
            else {
                let () = f.call(()).unwrap();
                r = lua.globals().get("r").unwrap();
            }

            return from_lua(&lua, r).to_record();
        });
    }
}
