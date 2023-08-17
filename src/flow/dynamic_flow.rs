use std::ops::Add;
use std::{sync::Arc, any::Any};
use std::fmt::Debug;

use serde_json::Value;

#[derive(Clone)]
pub struct FlowType(pub Arc<dyn Any + Send + Sync>);

impl Debug for FlowType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self.0.downcast_ref::<f64>() {
            return f.write_str(&v.to_string())
        }
        if let Some(v) = self.0.downcast_ref::<Value>() {
            return match v {
                Value::Null => todo!(),
                Value::Bool(b) => f.write_str(&b.to_string()),
                Value::Number(b) =>f.write_str(&b.to_string()),
                Value::String(b) =>f.write_str(&b.to_string()),
                _ => f.debug_tuple("FlowType").field(&self.0).finish(),
            }
        }
        f.debug_tuple("FlowType").field(&self.0).finish()
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
        if let Some(lhs) = self.0.downcast_ref::<i64>() {
            if let Some(rhs) = rhs.0.downcast_ref::<i64>() {
                return FlowType(Arc::new(lhs + rhs));
            }
        }
        if let Some(lhs) = self.0.downcast_ref::<i32>() {
            if let Some(rhs) = rhs.0.downcast_ref::<i32>() {
                return FlowType(Arc::new(lhs + rhs));
            }
        }
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
