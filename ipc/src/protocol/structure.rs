//! This module contains a compatibility structure with [`serde_json::Value`] for [`bincode`]
//! compatibility.

use serde::{Deserialize, Serialize};
use serde_json::Number;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum Structure {
    Null,
    Bool(bool),
    Number(StructureNumber),
    String(String),
    Array(Vec<Structure>),
    Object(BTreeMap<String, Structure>),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StructureNumber {
    PosInt(u64),
    NegInt(i64),
    Float(f64),
}

impl Structure {
    pub fn from_json(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(b),
            serde_json::Value::Number(n) => Self::Number(StructureNumber::from_json(n)),
            serde_json::Value::String(s) => Self::String(s),
            serde_json::Value::Array(a) => {
                Self::Array(a.into_iter().map(Self::from_json).collect())
            }
            serde_json::Value::Object(m) => Self::Object(
                m.into_iter()
                    .map(|(key, value)| (key, Self::from_json(value)))
                    .collect(),
            ),
        }
    }

    pub fn into_json(self) -> serde_json::Value {
        match self {
            Structure::Null => serde_json::Value::Null,
            Structure::Bool(b) => serde_json::Value::Bool(b),
            Structure::Number(n) => serde_json::Value::Number(n.into_json()),
            Structure::String(s) => serde_json::Value::String(s),
            Structure::Array(a) => {
                serde_json::Value::Array(a.into_iter().map(|v| v.into_json()).collect())
            }
            Structure::Object(m) => serde_json::Value::Object(
                m.into_iter()
                    .map(|(key, value)| (key, value.into_json()))
                    .collect(),
            ),
        }
    }
}

impl From<serde_json::Value> for Structure {
    fn from(value: serde_json::Value) -> Self {
        Self::from_json(value)
    }
}

impl From<Structure> for serde_json::Value {
    fn from(s: Structure) -> Self {
        s.into_json()
    }
}

impl StructureNumber {
    pub fn from_json(value: serde_json::Number) -> Self {
        if value.is_u64() {
            Self::PosInt(value.as_u64().unwrap())
        } else if value.is_i64() {
            Self::NegInt(value.as_i64().unwrap())
        } else if value.is_f64() {
            Self::Float(value.as_f64().unwrap())
        } else {
            unreachable!("Json value is neither a u64, i64 or f64")
        }
    }

    pub fn into_json(self) -> serde_json::Number {
        match self {
            StructureNumber::PosInt(v) => serde_json::Number::from(v),
            StructureNumber::NegInt(v) => serde_json::Number::from(v),
            StructureNumber::Float(v) => serde_json::Number::from_f64(v).unwrap(),
        }
    }
}

impl From<serde_json::Number> for StructureNumber {
    fn from(n: Number) -> Self {
        Self::from_json(n)
    }
}

impl From<StructureNumber> for serde_json::Number {
    fn from(s: StructureNumber) -> Self {
        s.into_json()
    }
}
