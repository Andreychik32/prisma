use datamodel::*;
use nullable::Nullable;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(tag = "stepType")]
pub enum MigrationStep {
    CreateModel(CreateModel),
    UpdateModel(UpdateModel),
    DeleteModel(DeleteModel),
    CreateField(CreateField),
    DeleteField(DeleteField),
    UpdateField(UpdateField),
    CreateEnum(CreateEnum),
    UpdateEnum(UpdateEnum),
    DeleteEnum(DeleteEnum),
}

pub trait WithDbName {
    fn db_name(&self) -> String;
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateModel {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_name: Option<String>,

    pub embedded: bool,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateModel {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_name: Option<String>,

    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "nullable::optional_nullable_deserialize"
    )]
    pub db_name: Option<Nullable<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedded: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteModel {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateField {
    pub model: String,

    pub name: String,

    #[serde(rename = "type")]
    pub tpe: FieldType,

    pub arity: FieldArity,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_created_at: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_updated_at: Option<bool>,

    pub is_unique: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<IdInfo>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scalar_list: Option<ScalarListStrategy>,
}

impl WithDbName for CreateField {
    fn db_name(&self) -> String {
        match self.db_name {
            Some(ref db_name) => db_name.clone(),
            None => self.name.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateField {
    pub model: String,

    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_name: Option<String>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub tpe: Option<FieldType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub arity: Option<FieldArity>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_name: Option<Nullable<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_created_at: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_updated_at: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Nullable<String>>, // fixme: change to behaviour

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Nullable<Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scalar_list: Option<Nullable<ScalarListStrategy>>,
}

impl UpdateField {
    pub fn is_any_option_set(&self) -> bool {
        self.new_name.is_some()
            || self.arity.is_some()
            || self.db_name.is_some()
            || self.is_created_at.is_some()
            || self.is_updated_at.is_some()
            || self.id.is_some()
            || self.default.is_some()
            || self.scalar_list.is_some()
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteField {
    pub model: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateEnum {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateEnum {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeleteEnum {
    pub name: String,
}