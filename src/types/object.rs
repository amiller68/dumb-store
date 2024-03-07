use std::collections::BTreeMap;

use libipld::Ipld;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::OffsetDateTime;

use crate::backend::Cid;
use crate::traits::Blockable;

#[derive(Debug, PartialEq, Eq)]
pub struct Object {
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    data: Cid,
    metadata: Value,
}

impl Default for Object {
    fn default() -> Self {
        Object {
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
            data: Cid::default(),
            metadata: Value::Null,
        }
    }
}

const OBJECT_CREATED_AT_LABEL: &str = "created_at";
const OBJECT_UPDATED_AT_LABEL: &str = "updated_at";
const OBJECT_DATA_LABEL: &str = "data";
const OBJECT_METADATA_LABEL: &str = "metadata";

impl Blockable for Object {
    type Error = ObjectIpldError;

    fn to_ipld(&self) -> Ipld {
        let mut map = BTreeMap::new();

        map.insert(
            OBJECT_CREATED_AT_LABEL.to_string(),
            Ipld::Integer(self.created_at().unix_timestamp_nanos()),
        );
        map.insert(
            OBJECT_UPDATED_AT_LABEL.to_string(),
            Ipld::Integer(self.updated_at().unix_timestamp_nanos()),
        );
        map.insert(
            OBJECT_DATA_LABEL.to_string(),
            Ipld::Link(self.data().clone()),
        );
        let metadata_string = self.metadata().to_string();
        map.insert(
            OBJECT_METADATA_LABEL.to_string(),
            Ipld::String(metadata_string),
        );
        Ipld::Map(map)
    }

    fn from_ipld(ipld: &Ipld) -> Result<Self, Self::Error> {
        let map = match ipld {
            Ipld::Map(map) => map,
            _ => return Err(ObjectIpldError::NotMap),
        };

        let created_at_int = match map.get(OBJECT_CREATED_AT_LABEL) {
            Some(Ipld::Integer(created_at)) => created_at.clone(),
            _ => {
                return Err(ObjectIpldError::MissingMapMember(
                    OBJECT_CREATED_AT_LABEL.to_string(),
                ))
            }
        };
        let created_at = OffsetDateTime::from_unix_timestamp_nanos(created_at_int)?;

        let updated_at_int = match map.get(OBJECT_UPDATED_AT_LABEL) {
            Some(Ipld::Integer(updated_at)) => updated_at.clone(),
            _ => {
                return Err(ObjectIpldError::MissingMapMember(
                    OBJECT_UPDATED_AT_LABEL.to_string(),
                ))
            }
        };
        let updated_at = OffsetDateTime::from_unix_timestamp_nanos(updated_at_int)?;

        let data = match map.get(OBJECT_DATA_LABEL) {
            Some(Ipld::Link(data)) => data.clone(),
            _ => {
                return Err(ObjectIpldError::MissingMapMember(
                    OBJECT_DATA_LABEL.to_string(),
                ))
            }
        };

        let metadata_string = match map.get(OBJECT_METADATA_LABEL) {
            Some(Ipld::String(metadata_string)) => metadata_string,
            _ => {
                return Err(ObjectIpldError::MissingMapMember(
                    OBJECT_METADATA_LABEL.to_string(),
                ))
            }
        };
        let metadata: Value = serde_json::from_str(&metadata_string)?;

        Ok(Self {
            created_at,
            updated_at,
            data,
            metadata,
        })
    }
}

impl Object {
    /* Getters */

    pub fn created_at(&self) -> &OffsetDateTime {
        &self.created_at
    }

    pub fn updated_at(&self) -> &OffsetDateTime {
        &self.updated_at
    }

    pub fn data(&self) -> &Cid {
        &self.data
    }

    pub fn metadata(&self) -> &Value {
        &self.metadata
    }

    /* Updaters */

    /// Update the data, metadata or both
    pub fn update(&mut self, data: Option<Cid>, metadata: Option<Value>) {
        self.updated_at = OffsetDateTime::now_utc();
        match data {
            Some(cid) => self.data = cid,
            None => {}
        }
        match metadata {
            Some(value) => self.metadata = value,
            None => {}
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectIpldError {
    #[error("invalid datetime: {0}")]
    InvalidDateTime(#[from] time::error::ComponentRange),
    #[error("missing map member: {0}")]
    MissingMapMember(String),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("ipld data is not map")]
    NotMap,
}
