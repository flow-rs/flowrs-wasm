use std::fmt::{Debug, Write};
use std::ops::Add;
use std::{any::Any, sync::Arc};

use serde_json::Value;

#[derive(Clone)]
pub struct FlowType(pub Arc<dyn Any + Send + Sync>);

macro_rules! try_dbg {
    ($self:expr, $f:expr, $ty:ident) => {
        if let Some(v) = $self.0.downcast_ref::<$ty>() {
            return $f.write_str(&v.to_string());
        }
    };
}

macro_rules! try_add {
    ($self:expr, $rhs:expr, $lhs_ty:ident, $rhs_ty:ident) => {
        if let Some(lhs) = $self.0.downcast_ref::<$lhs_ty>() {
            if let Some(rhs) = $rhs.0.downcast_ref::<$rhs_ty>() {
                return FlowType(Arc::new(lhs + rhs));
            }
        }
    };
}

impl Debug for FlowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        try_dbg!(self, f, i8);
        try_dbg!(self, f, i16);
        try_dbg!(self, f, i32);
        try_dbg!(self, f, i64);
        try_dbg!(self, f, i128);
        try_dbg!(self, f, u8);
        try_dbg!(self, f, u16);
        try_dbg!(self, f, u32);
        try_dbg!(self, f, u64);
        try_dbg!(self, f, u128);
        try_dbg!(self, f, String);
        try_dbg!(self, f, char);
        if let Some(v) = self.0.downcast_ref::<Vec<FlowType>>() {
            f.debug_list()
                .entries(v.iter())
                .finish()
                .expect("Couldn't parse Vec like Input/Output");
        }
        if let Some(v) = self.0.downcast_ref::<Value>() {
            return match v {
                Value::Null => todo!(),
                Value::Bool(b) => f.write_str(&b.to_string()),
                Value::Number(b) => f.write_str(&b.to_string()),
                Value::String(b) => f.write_str(&b.to_string()),
                _ => f.debug_tuple("FlowType").field(&self.0).finish(),
            };
        }
        f.debug_tuple("Unknown FlowType").field(&self.0).finish()
    }
}

impl From<Vec<FlowType>> for FlowType {
    fn from(value: Vec<FlowType>) -> Self {
        FlowType(Arc::new(value))
    }
}

// This implementation gives some control over which types should be
// addable throughout the entire flow. As of now only homogenious types
// allow addition.
// As the Properties of a Node can be any JSON value, the addition of
// such properties is limited to numbers (casted as float), lists and
// strings (both concatinated upon addition).
impl Add for FlowType {
    type Output = FlowType;

    fn add(self, rhs: Self) -> Self::Output {
        try_add!(self, rhs, i64, i64);
        try_add!(self, rhs, i32, i32);
        if let Some(lhs) = self.0.downcast_ref::<String>() {
            if let Some(rhs) = rhs.0.downcast_ref::<String>() {
                let mut res = lhs.clone();
                res.push_str(rhs);
                return FlowType(Arc::new(res));
            }
        }
        if let Some(lhs) = self.0.downcast_ref::<Value>() {
            if let Some(rhs) = rhs.0.downcast_ref::<Value>() {
                return match (lhs, rhs) {
                    (Value::Number(a), Value::Number(b)) => {
                        FlowType(Arc::new(a.as_f64().unwrap() + b.as_f64().unwrap()))
                    }
                    (Value::String(a), Value::String(b)) => {
                        let mut res = a.clone();
                        res.push_str(b);
                        FlowType(Arc::new(a.clone()))
                    }
                    (Value::Array(a), Value::Array(b)) => {
                        let mut res = a.clone();
                        res.append(b.to_owned().as_mut());
                        FlowType(Arc::new(a.clone()))
                    }
                    (a, b) => panic!(
                        "Addition of JSON values of type {:?} and {:?} is not supported.",
                        a, b
                    ),
                };
            }
        }
        panic!(
            "Addition not supported for type {:?} and {:?}.",
            self.type_id(),
            rhs.type_id()
        );
    }
}
