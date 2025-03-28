use std::collections::BTreeMap;
use std::io;
use std::sync::Arc;

use arrow::array::{ArrayRef, GenericStringBuilder, ListBuilder};
use arrow::datatypes::{DataType, Field as ArrowField};

use noodles::gff::record::attributes::field::Value as GffValue;
use noodles::gtf::record::attributes::Entry as GtfEntry;

/// An GXF attribute definition.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct AttributeDef {
    pub name: String,
    pub ty: AttributeType,
}

impl AttributeDef {
    pub fn get_arrow_field(&self) -> ArrowField {
        ArrowField::new(&self.name, self.ty.arrow_type(), true)
    }
}

impl TryFrom<(String, String)> for AttributeDef {
    type Error = io::Error;

    fn try_from(def: (String, String)) -> Result<Self, Self::Error> {
        let (name, ty) = def;
        let ty = match ty.to_lowercase().as_str() {
            "string" => Ok(AttributeType::String),
            "array" => Ok(AttributeType::Array),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Invalid attribute type: '{}'. Must be 'String' or 'Array'.",
                    ty
                ),
            )),
        }?;
        Ok(Self { name, ty })
    }
}

/// A mapping of native attribute field types to Arrow data types.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AttributeType {
    String,
    Array,
}

impl AttributeType {
    pub fn arrow_type(&self) -> DataType {
        match self {
            Self::String => DataType::Utf8,
            Self::Array => DataType::List(Arc::new(ArrowField::new("item", DataType::Utf8, true))),
        }
    }
}

/// Harmonizes GTF and GFF attribute values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    String(String),
    Array(Vec<String>),
}

impl From<GtfEntry> for AttributeValue {
    fn from(entry: GtfEntry) -> Self {
        Self::String(entry.value().to_string())
    }
}

impl<'a> From<&'a GffValue<'a>> for AttributeValue {
    fn from(value: &'a GffValue<'a>) -> Self {
        match value {
            GffValue::String(s) => Self::String(s.to_string()),
            GffValue::Array(a) => Self::Array(a.iter().map(|s| s.unwrap().to_string()).collect()),
        }
    }
}

/// A builder for Arrow arrays (columns) based on GXF attributes.
#[derive(Debug)]
pub enum AttributeBuilder {
    String(GenericStringBuilder<i32>),
    Array(ListBuilder<GenericStringBuilder<i32>>),
}

impl AttributeBuilder {
    pub fn new(attr_type: &AttributeType) -> Self {
        match attr_type {
            AttributeType::String => Self::String(GenericStringBuilder::<i32>::new()),
            AttributeType::Array => Self::Array(ListBuilder::<GenericStringBuilder<i32>>::new(
                GenericStringBuilder::<i32>::new(),
            )),
        }
    }

    pub fn append_null(&mut self) {
        match self {
            Self::String(builder) => builder.append_null(),
            Self::Array(builder) => builder.append(false),
        }
    }

    pub fn append_value(&mut self, value: &AttributeValue) -> io::Result<()> {
        match self {
            Self::String(builder) => match value {
                AttributeValue::String(v) => {
                    builder.append_value(v);
                    Ok(())
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Type mismatch: expected builder for {:?}, got {:?}",
                        value, self
                    ),
                )),
            },
            Self::Array(builder) => match value {
                AttributeValue::Array(array) => {
                    for value in array.iter() {
                        builder.values().append_value(value);
                    }
                    builder.append(true);
                    Ok(())
                }
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Type mismatch: expected builder for {:?}, got {:?}",
                        value, self
                    ),
                )),
            },
        }
    }

    pub fn finish(&mut self) -> ArrayRef {
        match self {
            Self::String(builder) => Arc::new(builder.finish()),
            Self::Array(builder) => Arc::new(builder.finish()),
        }
    }
}

/// A scanner to collect unique attribute definitions from a stream of GXF records.
pub struct AttributeScanner {
    attrs: BTreeMap<String, String>,
}

impl Default for AttributeScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeScanner {
    pub fn new() -> Self {
        Self {
            attrs: BTreeMap::new(),
        }
    }

    pub fn collect(&self) -> Vec<(String, String)> {
        let defs: Vec<(String, String)> = self
            .attrs
            .iter()
            .map(|(key, attr_type)| (key.clone(), attr_type.clone()))
            .collect();
        defs
    }
}

pub trait Push<T> {
    fn push(&mut self, record: T);
}

impl Push<noodles::gtf::Record> for AttributeScanner {
    fn push(&mut self, record: noodles::gtf::Record) {
        record.attributes().as_ref().iter().for_each(|entry| {
            let key = entry.key();
            self.attrs
                .entry(key.to_string())
                .or_insert_with(|| "String".to_string());
        });
    }
}

impl<'a> Push<noodles::gff::Record<'a>> for AttributeScanner {
    fn push(&mut self, record: noodles::gff::Record) {
        record.attributes().iter().for_each(|result| {
            let (key, value) = result.unwrap();
            self.attrs
                .entry(key.to_string())
                .or_insert_with(|| match value {
                    GffValue::String(_) => "String".to_string(),
                    GffValue::Array(_) => "Array".to_string(),
                });
        });
    }
}
