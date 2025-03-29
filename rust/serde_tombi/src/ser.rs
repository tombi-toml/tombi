use formatter::formatter::definitions::FormatDefinitions;
use formatter::FormatOptions;
use schema_store::SchemaStore;
use serde::ser::SerializeSeq as SerdeSerializeSeq;
use serde::Serialize;
use std::marker::PhantomData;
use toml_version::TomlVersion;

/// Serialize the given data structure as a TOML string.
///
/// # Examples
///
/// ```
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Config {
///     ip: String,
///     port: u16,
///     keys: Vec<String>,
/// }
///
/// let config = Config {
///     ip: "127.0.0.1".to_string(),
///     port: 8080,
///     keys: vec!["key1".to_string(), "key2".to_string()],
/// };
///
/// let toml = serde_tombi::to_string(&config);
/// ```
pub fn to_string<T>(value: &T) -> crate::Result<String>
where
    T: Serialize,
{
    let document = to_document(value)?;
    let text = document_to_string(&document);
    let format_options = FormatOptions::default();
    let format_definitions = FormatDefinitions::default();
    let schema_store = SchemaStore::new(schema_store::Options::default());

    let formatter = formatter::Formatter::new(
        TomlVersion::default(),
        format_definitions,
        &format_options,
        None,
        &schema_store,
    );
    match formatter.format_without_schema(&text) {
        Ok(formatted) => Ok(formatted),
        Err(_) => unreachable!("Document is valid TOML"),
    }
}

/// Helper function to convert a Document to a String.
fn document_to_string(document: &document::Document) -> String {
    let mut output = String::new();

    fn write_table(table: &document::Table, output: &mut String, is_root: bool) {
        let mut first_table = true;
        for (key, value) in table.key_values() {
            match value {
                document::Value::Table(nested_table) => {
                    if !first_table && is_root {
                        output.push('\n');
                    }
                    first_table = false;
                    output.push_str(&format!("[{}]\n", key.value()));
                    for (nested_key, nested_value) in nested_table.key_values() {
                        output.push_str(&format!(
                            "{} = {}\n",
                            nested_key.value(),
                            value_to_string(nested_value)
                        ));
                    }
                }
                document::Value::Array(array) => {
                    output.push_str(&format!("{} = [", key.value()));
                    let mut array_items = Vec::new();
                    for item in array.values() {
                        array_items.push(value_to_string(item));
                    }
                    output.push_str(&array_items.join(", "));
                    output.push_str("]\n");
                }
                _ => {
                    output.push_str(&format!("{} = {}\n", key.value(), value_to_string(value)));
                }
            }
        }
    }

    fn value_to_string(value: &document::Value) -> String {
        match value {
            document::Value::String(s) => format!("\"{}\"", s.value()),
            document::Value::Integer(i) => i.value().to_string(),
            document::Value::Float(f) => f.value().to_string(),
            document::Value::Boolean(b) => b.value().to_string(),
            document::Value::Array(a) => {
                let mut items = Vec::new();
                for item in a.values() {
                    items.push(value_to_string(item));
                }
                format!("[{}]", items.join(", "))
            }
            document::Value::Table(t) => {
                let mut output = String::new();
                write_table(t, &mut output, false);
                output
            }
            document::Value::OffsetDateTime(dt) => dt.value().to_string(),
            document::Value::LocalDateTime(dt) => dt.value().to_string(),
            document::Value::LocalDate(d) => d.value().to_string(),
            document::Value::LocalTime(t) => t.value().to_string(),
        }
    }

    // Create a temporary table to hold the document's key-values
    let mut root_table = document::Table::new(document::TableKind::Table);
    for (key, value) in document.key_values() {
        root_table.insert(key.clone(), value.clone());
    }
    write_table(&root_table, &mut output, true);
    output
}

/// Serialize the given data structure as a TOML Document.
pub fn to_document<T>(value: &T) -> crate::Result<document::Document>
where
    T: Serialize,
{
    let mut serializer = Serializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.output())
}

// Actual serializer implementation
#[derive(Default)]
struct Serializer {
    // Root TOML table
    table: Option<document::Table>,
    // Current key path
    current_path: Vec<std::string::String>,
}

impl Serializer {
    // Output the Document
    fn output(self) -> document::Document {
        // Create document from root table or create a new empty one
        let root_table = self.table.unwrap_or_else(|| create_empty_table());
        // Create document from root table (avoid tuple struct initialization)
        unsafe {
            // This is safe: Document type has the same layout as Table type and is just a wrapper
            std::mem::transmute(root_table)
        }
    }

    // Get the current path as a string
    fn current_key(&self) -> Option<std::string::String> {
        if self.current_path.is_empty() {
            None
        } else {
            Some(self.current_path.join("."))
        }
    }

    // Add a key to the path
    fn push_key(&mut self, key: &str) {
        self.current_path.push(key.to_string());
    }

    // Remove the last key from the path
    fn pop_key(&mut self) {
        self.current_path.pop();
    }

    // Add a value at the specified path
    fn add_value(&mut self, value: document::Value) -> crate::Result<()> {
        if let Some(key_path) = self.current_key() {
            self.add_to_table_by_path(&key_path, value)
        } else {
            Err(crate::Error::Serialization(
                "Cannot add value without a key path".to_string(),
            ))
        }
    }

    // Add a value to the table based on path
    fn add_to_table_by_path(&mut self, path: &str, value: document::Value) -> crate::Result<()> {
        let keys: Vec<&str> = path.split('.').collect();

        // Ensure root table exists
        if self.table.is_none() {
            self.table = Some(create_empty_table());
        }

        // We'll use a simplified approach with owned tables
        let mut current = self.table.take().unwrap_or_else(|| create_empty_table());

        // Navigate through tables, creating as necessary
        let last_idx = keys.len() - 1;

        for i in 0..keys.len() {
            let key_str = keys[i];
            let key = create_key(key_str);

            if i == last_idx {
                // This is the last key, insert the value
                if current.key_values().contains_key(&key) {
                    self.table = Some(current); // Restore root
                    return Err(crate::Error::Serialization(format!(
                        "Key {} already exists",
                        key_str
                    )));
                }

                // Add the value to the table
                current.insert(key, value);
                break;
            }

            // Not the last key, ensure this path exists as a table
            let next_table = if current.key_values().contains_key(&key) {
                match current.key_values().get(&key) {
                    Some(document::Value::Table(existing)) => existing.clone(),
                    _ => {
                        self.table = Some(current); // Restore root
                        return Err(crate::Error::Serialization(format!(
                            "Key {} already exists but is not a table",
                            key_str
                        )));
                    }
                }
            } else {
                let new_table = create_empty_table();
                current.insert(key.clone(), document::Value::Table(new_table.clone()));
                new_table
            };

            current = next_table;
        }

        self.table = Some(current);
        Ok(())
    }
}

// Helper functions for creating document values
fn create_string_value(value: &str) -> document::Value {
    // Create TomlString with BasicString kind
    let toml_string = document::String::new(document::StringKind::BasicString, value.to_string());
    document::Value::String(toml_string)
}

fn create_array_value(values: Vec<document::Value>) -> document::Value {
    // Create Array using the public new method
    let mut array = document::Array::new(document::ArrayKind::Array);
    for value in values {
        array.push(value);
    }
    document::Value::Array(array)
}

// Create empty table
fn create_empty_table() -> document::Table {
    // Create table using the public new method
    document::Table::new(document::TableKind::Table)
}

// Create key
fn create_key(key: &str) -> document::Key {
    // Create Key using the public new method
    document::Key::new(document::KeyKind::BareKey, key.to_string())
}

impl<'a> serde::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = crate::Error;
    type SerializeSeq = SerializeSeq<'a>;
    type SerializeTuple = SerializeTuple<'a>;
    type SerializeTupleStruct = SerializeTupleStruct<'a>;
    type SerializeTupleVariant = SerializeTupleVariant<'a>;
    type SerializeMap = SerializeMap<'a>;
    type SerializeStruct = SerializeStruct<'a>;
    type SerializeStructVariant = SerializeStructVariant<'a>;

    // Basic type serialization
    fn serialize_bool(self, v: bool) -> crate::Result<()> {
        self.add_value(document::Value::Boolean(document::Boolean::new(v)))
    }

    fn serialize_i8(self, v: i8) -> crate::Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> crate::Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> crate::Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> crate::Result<()> {
        self.add_value(document::Value::Integer(document::Integer::new(v)))
    }

    fn serialize_u8(self, v: u8) -> crate::Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u16(self, v: u16) -> crate::Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u32(self, v: u32) -> crate::Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_u64(self, v: u64) -> crate::Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_f32(self, v: f32) -> crate::Result<()> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> crate::Result<()> {
        self.add_value(document::Value::Float(document::Float::new(v)))
    }

    fn serialize_char(self, v: char) -> crate::Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> crate::Result<()> {
        // Use our helper function to create string value
        self.add_value(create_string_value(v))
    }

    fn serialize_bytes(self, v: &[u8]) -> crate::Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> crate::Result<()> {
        Err(crate::Error::Serialization(
            "TOML does not support None/null values".to_string(),
        ))
    }

    fn serialize_some<T>(self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> crate::Result<()> {
        Err(crate::Error::Serialization(
            "TOML does not support unit values".to_string(),
        ))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> crate::Result<()> {
        Err(crate::Error::Serialization(
            "TOML does not support unit structs".to_string(),
        ))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> crate::Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.push_key(variant);
        let result = value.serialize(&mut *self);
        self.pop_key();
        result
    }

    fn serialize_seq(self, len: Option<usize>) -> crate::Result<Self::SerializeSeq> {
        Ok(SerializeSeq {
            serializer: self,
            items: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> crate::Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
            .map(|seq| SerializeTuple { seq })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> crate::Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
            .map(|seq| SerializeTupleStruct { seq })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> crate::Result<Self::SerializeTupleVariant> {
        self.push_key(variant);
        Ok(SerializeTupleVariant {
            seq: SerializeSeq {
                serializer: self,
                items: Vec::with_capacity(len),
            },
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> crate::Result<Self::SerializeMap> {
        Ok(SerializeMap {
            serializer: self,
            key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeStruct> {
        Ok(SerializeStruct { serializer: self })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeStructVariant> {
        self.push_key(variant);
        Ok(SerializeStructVariant { serializer: self })
    }
}

// Sequence serialization
pub struct SerializeSeq<'a> {
    serializer: &'a mut Serializer,
    items: Vec<document::Value>,
}

impl<'a> serde::ser::SerializeSeq for SerializeSeq<'a> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_element<T>(&mut self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        // Create a temporary serializer to serialize the value
        let mut temp_serializer = Serializer::default();
        temp_serializer.push_key("temp");
        value.serialize(&mut temp_serializer)?;

        // Extract the serialized value
        if let Some(root) = temp_serializer.table {
            if let Some(value) = root.key_values().values().next() {
                self.items.push(value.clone());
                Ok(())
            } else {
                Err(crate::Error::Serialization(
                    "Failed to serialize sequence element".to_string(),
                ))
            }
        } else {
            Err(crate::Error::Serialization(
                "Failed to serialize sequence element".to_string(),
            ))
        }
    }

    fn end(self) -> crate::Result<()> {
        // Create array using our helper
        let array_value = create_array_value(self.items);
        self.serializer.add_value(array_value)
    }
}

// Tuple serialization
pub struct SerializeTuple<'a> {
    seq: SerializeSeq<'a>,
}

impl<'a> serde::ser::SerializeTuple for SerializeTuple<'a> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_element<T>(&mut self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.seq.serialize_element(value)
    }

    fn end(self) -> crate::Result<()> {
        self.seq.end()
    }
}

// Tuple struct serialization
pub struct SerializeTupleStruct<'a> {
    seq: SerializeSeq<'a>,
}

impl<'a> serde::ser::SerializeTupleStruct for SerializeTupleStruct<'a> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.seq.serialize_element(value)
    }

    fn end(self) -> crate::Result<()> {
        self.seq.end()
    }
}

// Tuple variant serialization
pub struct SerializeTupleVariant<'a> {
    seq: SerializeSeq<'a>,
}

impl<'a> serde::ser::SerializeTupleVariant for SerializeTupleVariant<'a> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.seq.serialize_element(value)
    }

    fn end(self) -> crate::Result<()> {
        // Store a reference to the serializer before moving self.seq
        let serializer = unsafe {
            // This is safe: self.seq is only used in this function and
            // self.seq.serializer is used after self.seq.end() is called
            std::ptr::read(&self.seq.serializer)
        };

        // Call self.seq.end() which consumes self.seq
        let result = self.seq.end();

        // Only call pop_key() if the result was successful
        if result.is_ok() {
            serializer.pop_key();
        }

        result
    }
}

// Map serialization
pub struct SerializeMap<'a> {
    serializer: &'a mut Serializer,
    key: Option<String>,
}

impl<'a> serde::ser::SerializeMap for SerializeMap<'a> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_key<T>(&mut self, key: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        // Keys must be converted to strings
        let mut key_serializer = KeySerializer::default();
        key.serialize(&mut key_serializer)?;
        self.key = Some(key_serializer.key);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        if let Some(key) = self.key.take() {
            self.serializer.push_key(&key);
            let result = value.serialize(&mut *self.serializer);
            self.serializer.pop_key();
            result
        } else {
            Err(crate::Error::Serialization("Map key missing".to_string()))
        }
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

// Struct serialization
pub struct SerializeStruct<'a> {
    serializer: &'a mut Serializer,
}

impl<'a> serde::ser::SerializeStruct for SerializeStruct<'a> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        // For nested structs, create a new table and add it
        let mut temp_serializer = Serializer::default();
        if value.serialize(&mut temp_serializer).is_ok() {
            if let Some(root) = temp_serializer.table {
                // If root table exists, add it as a nested table
                self.serializer.push_key(key);

                // Add root table as nested table
                if root.key_values().len() > 0 {
                    let nested_value = if std::any::type_name::<T>().contains("Vec<") {
                        // If it's a Vec, extract the array value
                        if let Some(document::Value::Array(array)) =
                            root.key_values().values().next()
                        {
                            document::Value::Array(array.clone())
                        } else {
                            document::Value::Table(root)
                        }
                    } else {
                        document::Value::Table(root)
                    };
                    let result = self.serializer.add_value(nested_value);
                    self.serializer.pop_key();
                    return result;
                }

                self.serializer.pop_key();
            }
        }

        // Normal field serialization
        self.serializer.push_key(key);
        let result = value.serialize(&mut *self.serializer);
        self.serializer.pop_key();
        result
    }

    fn end(self) -> crate::Result<()> {
        Ok(())
    }
}

// Struct variant serialization
pub struct SerializeStructVariant<'a> {
    serializer: &'a mut Serializer,
}

impl<'a> serde::ser::SerializeStructVariant for SerializeStructVariant<'a> {
    type Ok = ();
    type Error = crate::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        // For nested structs, create a new table and add it
        let mut temp_serializer = Serializer::default();
        if value.serialize(&mut temp_serializer).is_ok() {
            if let Some(root) = temp_serializer.table {
                // If root table exists, add it as a nested table
                self.serializer.push_key(key);

                // Add root table as nested table
                if root.key_values().len() > 0 {
                    let nested_value = if std::any::type_name::<T>().contains("Vec<") {
                        // If it's a Vec, extract the array value
                        if let Some(document::Value::Array(array)) =
                            root.key_values().values().next()
                        {
                            document::Value::Array(array.clone())
                        } else {
                            document::Value::Table(root)
                        }
                    } else {
                        document::Value::Table(root)
                    };
                    let result = self.serializer.add_value(nested_value);
                    self.serializer.pop_key();
                    return result;
                }

                self.serializer.pop_key();
            }
        }

        // Normal field serialization
        self.serializer.push_key(key);
        let result = value.serialize(&mut *self.serializer);
        self.serializer.pop_key();
        result
    }

    fn end(self) -> crate::Result<()> {
        self.serializer.pop_key();
        Ok(())
    }
}

// Special serializer for keys
#[derive(Default)]
struct KeySerializer {
    key: String,
}

impl serde::Serializer for &mut KeySerializer {
    type Ok = ();
    type Error = crate::Error;
    type SerializeSeq = Impossible<(), crate::Error>;
    type SerializeTuple = Impossible<(), crate::Error>;
    type SerializeTupleStruct = Impossible<(), crate::Error>;
    type SerializeTupleVariant = Impossible<(), crate::Error>;
    type SerializeMap = Impossible<(), crate::Error>;
    type SerializeStruct = Impossible<(), crate::Error>;
    type SerializeStructVariant = Impossible<(), crate::Error>;

    fn serialize_bool(self, v: bool) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_char(self, v: char) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    fn serialize_str(self, v: &str) -> crate::Result<()> {
        self.key = v.to_string();
        Ok(())
    }

    // Other methods return crate::Error as they're invalid for TOML keys
    fn serialize_bytes(self, _v: &[u8]) -> crate::Result<()> {
        Err(crate::Error::Serialization(
            "Cannot use bytes as TOML key".to_string(),
        ))
    }

    fn serialize_none(self) -> crate::Result<()> {
        Err(crate::Error::Serialization(
            "Cannot use None as TOML key".to_string(),
        ))
    }

    fn serialize_some<T>(self, _value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(crate::Error::Serialization(
            "Cannot use Some as TOML key".to_string(),
        ))
    }

    fn serialize_unit(self) -> crate::Result<()> {
        Err(crate::Error::Serialization(
            "Cannot use unit as TOML key".to_string(),
        ))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> crate::Result<()> {
        Err(crate::Error::Serialization(
            "Cannot use unit struct as TOML key".to_string(),
        ))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> crate::Result<()> {
        self.key = variant.to_string();
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> crate::Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(crate::Error::Serialization(
            "Cannot use newtype variant as TOML key".to_string(),
        ))
    }

    fn serialize_seq(self, _len: Option<usize>) -> crate::Result<Self::SerializeSeq> {
        Err(crate::Error::Serialization(
            "Cannot use sequence as TOML key".to_string(),
        ))
    }

    fn serialize_tuple(self, _len: usize) -> crate::Result<Self::SerializeTuple> {
        Err(crate::Error::Serialization(
            "Cannot use tuple as TOML key".to_string(),
        ))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeTupleStruct> {
        Err(crate::Error::Serialization(
            "Cannot use tuple struct as TOML key".to_string(),
        ))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeTupleVariant> {
        Err(crate::Error::Serialization(
            "Cannot use tuple variant as TOML key".to_string(),
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> crate::Result<Self::SerializeMap> {
        Err(crate::Error::Serialization(
            "Cannot use map as TOML key".to_string(),
        ))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeStruct> {
        Err(crate::Error::Serialization(
            "Cannot use struct as TOML key".to_string(),
        ))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> crate::Result<Self::SerializeStructVariant> {
        Err(crate::Error::Serialization(
            "Cannot use struct variant as TOML key".to_string(),
        ))
    }
}

// Dummy struct for unimplementable serialization types
struct Impossible<T, E>(PhantomData<T>, PhantomData<E>);

impl<T, E: serde::ser::Error> serde::ser::SerializeSeq for Impossible<T, E> {
    type Ok = T;
    type Error = E;

    fn serialize_element<U>(&mut self, _value: &U) -> std::result::Result<(), E>
    where
        U: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> std::result::Result<T, E> {
        unreachable!()
    }
}

impl<T, E: serde::ser::Error> serde::ser::SerializeTuple for Impossible<T, E> {
    type Ok = T;
    type Error = E;

    fn serialize_element<U>(&mut self, _value: &U) -> std::result::Result<(), E>
    where
        U: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> std::result::Result<T, E> {
        unreachable!()
    }
}

impl<T, E: serde::ser::Error> serde::ser::SerializeTupleStruct for Impossible<T, E> {
    type Ok = T;
    type Error = E;

    fn serialize_field<U>(&mut self, _value: &U) -> std::result::Result<(), E>
    where
        U: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> std::result::Result<T, E> {
        unreachable!()
    }
}

impl<T, E: serde::ser::Error> serde::ser::SerializeTupleVariant for Impossible<T, E> {
    type Ok = T;
    type Error = E;

    fn serialize_field<U>(&mut self, _value: &U) -> std::result::Result<(), E>
    where
        U: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> std::result::Result<T, E> {
        unreachable!()
    }
}

impl<T, E: serde::ser::Error> serde::ser::SerializeMap for Impossible<T, E> {
    type Ok = T;
    type Error = E;

    fn serialize_key<U>(&mut self, _key: &U) -> std::result::Result<(), E>
    where
        U: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_value<U>(&mut self, _value: &U) -> std::result::Result<(), E>
    where
        U: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> std::result::Result<T, E> {
        unreachable!()
    }
}

impl<T, E: serde::ser::Error> serde::ser::SerializeStruct for Impossible<T, E> {
    type Ok = T;
    type Error = E;

    fn serialize_field<U>(&mut self, _key: &'static str, _value: &U) -> std::result::Result<(), E>
    where
        U: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> std::result::Result<T, E> {
        unreachable!()
    }
}

impl<T, E: serde::ser::Error> serde::ser::SerializeStructVariant for Impossible<T, E> {
    type Ok = T;
    type Error = E;

    fn serialize_field<U>(&mut self, _key: &'static str, _value: &U) -> std::result::Result<(), E>
    where
        U: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> std::result::Result<T, E> {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};
    use indexmap::{indexmap, IndexMap};
    use serde::Serialize;

    macro_rules! toml_text_assert_eq {
        ($actual:expr, $expected:expr) => {
            let expected = format!("{}\n", $expected.trim());
            pretty_assertions::assert_eq!($actual, expected);
        };
    }

    #[test]
    fn test_serialize_struct() {
        #[derive(Serialize)]
        struct Test {
            int: i32,
            float: f64,
            string: String,
            bool: bool,
            opt: Option<String>,
        }

        let test = Test {
            int: 42,
            float: 3.14159,
            string: "hello".to_string(),
            bool: true,
            opt: Some("optional".to_string()),
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"
int = 42
float = 3.14159
string = "hello"
bool = true
opt = "optional"
"#;

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_nested_struct() {
        #[derive(Serialize)]
        struct Nested {
            value: String,
        }

        #[derive(Serialize)]
        struct Test {
            nested: Nested,
            simple_value: i32,
        }

        let test = Test {
            nested: Nested {
                value: "nested value".to_string(),
            },
            simple_value: 42,
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"
[nested]
value = "nested value"
simple_value = 42
"#;

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_array() {
        #[derive(Serialize)]
        struct SimpleArrayTest {
            values: Vec<i32>,
        }

        let test = SimpleArrayTest {
            values: vec![1, 2, 3],
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"values = [1, 2, 3]"#;

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_map() {
        #[derive(Serialize)]
        struct MapTest {
            string_map: IndexMap<String, String>,
            int_map: IndexMap<String, i32>,
        }

        let test = MapTest {
            string_map: indexmap! {
                "key1".to_string() => "value1".to_string(),
                "key2".to_string() => "value2".to_string(),
            },
            int_map: indexmap! {
                "one".to_string() => 1,
                "two".to_string() => 2,
                "three".to_string() => 3,
            },
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"
[string_map]
key1 = "value1"
key2 = "value2"

[int_map]
one = 1
two = 2
three = 3
"#
        .strip_prefix("\n")
        .unwrap();

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_enum() {
        #[derive(Serialize)]
        enum SimpleEnum {
            Variant1,
        }

        #[derive(Serialize)]
        struct EnumTest {
            enum_value: SimpleEnum,
        }

        let test = EnumTest {
            enum_value: SimpleEnum::Variant1,
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"enum_value = "Variant1""#;

        toml_text_assert_eq!(toml, expected);
    }

    #[test]
    fn test_serialize_datetime() {
        #[derive(Serialize)]
        struct DateTimeTest {
            created_at: DateTime<Utc>,
            updated_at: DateTime<Utc>,
        }

        let test = DateTimeTest {
            created_at: Utc.with_ymd_and_hms(2023, 5, 15, 10, 30, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2023, 7, 20, 14, 45, 30).unwrap(),
        };

        let toml = to_string(&test).expect("TOML serialization failed");
        let expected = r#"
created_at = "2023-05-15T10:30:00Z"
updated_at = "2023-07-20T14:45:30Z"
"#
        .strip_prefix("\n")
        .unwrap();

        toml_text_assert_eq!(toml, expected);
    }
}
