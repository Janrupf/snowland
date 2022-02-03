//! Helper to convert [`snowland_ipc::protocol::structure::Structure`] to
//! [`nativeshell::codec::Value`] and back.

use nativeshell::codec::Value;
use snowland_ipc::protocol::structure::{Structure, StructureNumber};

use thiserror::Error;

pub fn structure_to_value(s: Structure) -> Value {
    match s {
        Structure::Null => Value::Null,
        Structure::Bool(b) => Value::Bool(b),
        Structure::Number(n) => structure_number_to_value(n),
        Structure::String(s) => Value::String(s),
        Structure::Array(a) => Value::List(a.into_iter().map(structure_to_value).collect()),
        Structure::Object(m) => Value::Map(
            m.into_iter()
                .map(|(key, value)| (Value::String(key), structure_to_value(value)))
                .collect(),
        ),
    }
}

pub fn structure_number_to_value(n: StructureNumber) -> Value {
    match n {
        StructureNumber::PosInt(u) => Value::I64(u as _),
        StructureNumber::NegInt(i) => Value::I64(i),
        StructureNumber::Float(f) => Value::F64(f),
    }
}

pub fn value_to_structure(v: Value) -> Structure {
    match v {
        Value::Null => Structure::Null,
        Value::Bool(b) => Structure::Bool(b),
        Value::I64(i) => Structure::Number(StructureNumber::NegInt(i)),
        Value::F64(f) => Structure::Number(StructureNumber::Float(f)),
        Value::String(s) => Structure::String(s),
        Value::U8List(l) => Structure::Array(vec_num_convert_unsigned(l)),
        Value::I32List(l) => Structure::Array(vec_num_convert_signed(l)),
        Value::I64List(l) => Structure::Array(vec_num_convert_signed(l)),
        Value::F64List(l) => Structure::Array(vec_num_convert_float(l)),
        Value::List(l) => Structure::Array(l.into_iter().map(value_to_structure).collect()),
        Value::Map(m) => Structure::Object(
            m.into_iter()
                .map(|(k, v)| (value_to_key(k), value_to_structure(v)))
                .collect(),
        ),
    }
}

fn value_to_key(v: Value) -> String {
    match v {
        Value::Null => String::from("null"),
        Value::Bool(b) => b.to_string(),
        Value::I64(i) => i.to_string(),
        Value::F64(f) => f.to_string(),
        Value::String(s) => s,
        Value::U8List(l) => vec_convert_string(l),
        Value::I32List(l) => vec_convert_string(l),
        Value::I64List(l) => vec_convert_string(l),
        Value::F64List(l) => vec_convert_string(l),
        Value::List(l) => l
            .into_iter()
            .map(value_to_key)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Map(m) => m
            .into_iter()
            .map(|(k, v)| format!("{} => {}", value_to_key(k), value_to_key(v)))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn vec_num_convert_unsigned<F>(input: Vec<F>) -> Vec<Structure>
where
    F: Into<u64>,
{
    input
        .into_iter()
        .map(|v| Structure::Number(StructureNumber::PosInt(v.into())))
        .collect()
}

fn vec_num_convert_signed<F>(input: Vec<F>) -> Vec<Structure>
where
    F: Into<i64>,
{
    input
        .into_iter()
        .map(|v| Structure::Number(StructureNumber::NegInt(v.into())))
        .collect()
}

fn vec_num_convert_float<F>(input: Vec<F>) -> Vec<Structure>
where
    F: Into<f64>,
{
    input
        .into_iter()
        .map(|v| Structure::Number(StructureNumber::Float(v.into())))
        .collect()
}

fn vec_convert_string<F>(input: Vec<F>) -> String
where
    F: std::fmt::Display,
{
    input
        .into_iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug, Error)]
pub enum DartConvertError {}
