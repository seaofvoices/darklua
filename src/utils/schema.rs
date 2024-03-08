use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};

pub(crate) fn string_literal(value: &str) -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
        enum_values: Some(vec![serde_json::Value::from(value)]),
        ..Default::default()
    })
}

pub(crate) fn string() -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
        ..Default::default()
    })
}

pub(crate) fn bool() -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(SingleOrVec::Single(InstanceType::Boolean.into())),
        ..Default::default()
    })
}

pub(crate) fn string_enum(value: impl IntoIterator<Item = &'static str>) -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(SingleOrVec::Single(InstanceType::String.into())),
        enum_values: Some(value.into_iter().map(serde_json::Value::from).collect()),
        ..Default::default()
    })
}

pub(crate) fn string_array() -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(SingleOrVec::Vec(vec![InstanceType::String.into()])),
        ..Default::default()
    })
}

pub(crate) fn any() -> Schema {
    Schema::Bool(true)
}

pub(crate) fn one_of(schemas: Vec<Schema>) -> Schema {
    let mut object = SchemaObject {
        ..Default::default()
    };
    object.subschemas().one_of = Some(schemas);
    Schema::Object(object)
}

pub(crate) fn with_default_value(
    mut schema: Schema,
    value: impl Into<serde_json::Value>,
) -> Schema {
    if let Schema::Object(object) = &mut schema {
        object.metadata().default.get_or_insert(value.into());
    }
    schema
}

pub(crate) fn object(
    properties: impl IntoIterator<Item = (&'static str, Schema)>,
    required: impl IntoIterator<Item = &'static str>,
) -> Schema {
    let mut object = SchemaObject {
        instance_type: Some(SingleOrVec::Single(InstanceType::Object.into())),
        ..Default::default()
    };
    for (property, property_schema) in properties {
        object
            .object()
            .properties
            .insert(property.to_owned(), property_schema);
    }
    for property in required {
        object.object().required.insert(property.to_owned());
    }
    Schema::Object(object)
}
