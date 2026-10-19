use serde::{
    Serialize,
    de::{DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Map, Number, Value};
use sha2::{Digest, Sha256};
use std::fmt;

pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub fn digest<T: Serialize>(domain: &str, value: &T) -> Result<String, String> {
    let mut bytes = format!("oathkeeper:{domain}:v1\0").into_bytes();
    bytes.extend(canonical_json(value)?);
    Ok(sha256(&bytes))
}
/// RFC 8785's integer-only subset. Exact application numbers are strings.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    fn encode(v: &Value, out: &mut String) -> Result<(), String> {
        match v {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::String(s) => out.push_str(&serde_json::to_string(s).map_err(|e| e.to_string())?),
            Value::Number(n) => {
                let good = n
                    .as_i64()
                    .is_some_and(|n| n.unsigned_abs() <= 9_007_199_254_740_991)
                    || n.as_u64().is_some_and(|n| n <= 9_007_199_254_740_991);
                if !good {
                    return Err(
                        "wire numbers must be safe integers; encode exact values as strings".into(),
                    );
                }
                out.push_str(&n.to_string());
            }
            Value::Array(a) => {
                out.push('[');
                for (i, v) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',')
                    }
                    encode(v, out)?
                }
                out.push(']')
            }
            Value::Object(m) => {
                let mut keys: Vec<_> = m.keys().collect();
                keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
                out.push('{');
                for (i, k) in keys.into_iter().enumerate() {
                    if i > 0 {
                        out.push(',')
                    }
                    out.push_str(&serde_json::to_string(k).map_err(|e| e.to_string())?);
                    out.push(':');
                    encode(&m[k], out)?
                }
                out.push('}');
            }
        }
        Ok(())
    }
    let v = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let mut out = String::new();
    encode(&v, &mut out)?;
    Ok(out.into_bytes())
}

struct Strict {
    depth: usize,
    max_depth: usize,
    max_items: usize,
}
impl<'de> DeserializeSeed<'de> for Strict {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        if self.depth > self.max_depth {
            return Err(serde::de::Error::custom("JSON nesting limit exceeded"));
        }
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Strict {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded JSON without duplicate keys")
    }
    fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Value, E> {
        Ok(Value::Number(Number::from(v)))
    }
    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Value, E> {
        Ok(Value::Number(Number::from(v)))
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Value, E> {
        Err(E::custom(
            "JSON floating-point numbers are not supported; use exact value strings",
        ))
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.into()))
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
        let mut v = Vec::new();
        while let Some(x) = a.next_element_seed(Strict {
            depth: self.depth + 1,
            ..self
        })? {
            if v.len() >= self.max_items {
                return Err(serde::de::Error::custom("JSON collection limit exceeded"));
            }
            v.push(x)
        }
        Ok(Value::Array(v))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
        let mut m = Map::new();
        while let Some(k) = a.next_key::<String>()? {
            if m.contains_key(&k) {
                return Err(serde::de::Error::custom(format!("duplicate JSON key {k}")));
            }
            if m.len() >= self.max_items {
                return Err(serde::de::Error::custom("JSON object limit exceeded"));
            }
            let v = a.next_value_seed(Strict {
                depth: self.depth + 1,
                ..self
            })?;
            m.insert(k, v);
        }
        Ok(Value::Object(m))
    }
}
pub fn read_json<T: DeserializeOwned>(
    bytes: &[u8],
    limits: &crate::ExecutionLimits,
) -> Result<T, String> {
    crate::validate_limits(limits)?;
    if bytes.len() > limits.input_bytes {
        return Err("JSON byte limit exceeded".into());
    }
    let mut d = serde_json::Deserializer::from_slice(bytes);
    let v = Strict {
        depth: 0,
        max_depth: limits.nesting,
        max_items: limits.trace_nodes.max(limits.collection_entries),
    }
    .deserialize(&mut d)
    .map_err(|e| e.to_string())?;
    d.end().map_err(|e| e.to_string())?;
    serde_json::from_value(v).map_err(|e| e.to_string())
}
