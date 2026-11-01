use crate::Error;
use crate::location_key::{LOCATION_KEY_PROPERTY, LocationKey};
use crate::models::{
    ObjectLabel, Resource, ResourceExt, ResourceIdent, ResourceName, ResourceRef, Table,
};

use super::tables::v1::TableSummary;

/// Project-specific alias for `resource_store::Object` with our generated `ObjectLabel`.
pub type Object = olai_store::Object<ObjectLabel>;

impl ResourceExt for Object {
    fn resource_name(&self) -> ResourceName {
        self.name.clone()
    }

    fn resource_ref(&self) -> ResourceRef {
        ResourceRef::Uuid(self.id)
    }

    fn resource_ident(&self) -> ResourceIdent {
        self.label.to_ident(self.id)
    }
}

impl ResourceExt for Resource {
    fn resource_name(&self) -> ResourceName {
        match self {
            Resource::Agent(obj) => obj.resource_name(),
            Resource::AgentSkill(obj) => obj.resource_name(),
            Resource::Share(obj) => obj.resource_name(),
            Resource::Credential(obj) => obj.resource_name(),
            Resource::Catalog(obj) => obj.resource_name(),
            Resource::Schema(obj) => obj.resource_name(),
            Resource::Table(obj) => obj.resource_name(),
            Resource::ExternalLocation(obj) => obj.resource_name(),
            Resource::Recipient(obj) => obj.resource_name(),
            Resource::Provider(obj) => obj.resource_name(),
            Resource::Column(obj) => obj.resource_name(),
            Resource::Volume(obj) => obj.resource_name(),
            Resource::Function(obj) => obj.resource_name(),
            Resource::TagPolicy(obj) => obj.resource_name(),
            Resource::StagingTable(obj) => obj.resource_name(),
            Resource::PolicyInfo(obj) => obj.resource_name(),
            Resource::RegisteredModel(obj) => obj.resource_name(),
            Resource::ModelVersion(obj) => obj.resource_name(),
        }
    }

    fn resource_ref(&self) -> ResourceRef {
        match self {
            Resource::Agent(obj) => obj.resource_ref(),
            Resource::AgentSkill(obj) => obj.resource_ref(),
            Resource::Share(obj) => obj.resource_ref(),
            Resource::Credential(obj) => obj.resource_ref(),
            Resource::Catalog(obj) => obj.resource_ref(),
            Resource::Schema(obj) => obj.resource_ref(),
            Resource::Table(obj) => obj.resource_ref(),
            Resource::ExternalLocation(obj) => obj.resource_ref(),
            Resource::Recipient(obj) => obj.resource_ref(),
            Resource::Provider(obj) => obj.resource_ref(),
            Resource::Column(obj) => obj.resource_ref(),
            Resource::Volume(obj) => obj.resource_ref(),
            Resource::Function(obj) => obj.resource_ref(),
            Resource::TagPolicy(obj) => obj.resource_ref(),
            Resource::StagingTable(obj) => obj.resource_ref(),
            Resource::PolicyInfo(obj) => obj.resource_ref(),
            Resource::RegisteredModel(obj) => obj.resource_ref(),
            Resource::ModelVersion(obj) => obj.resource_ref(),
        }
    }

    fn resource_ident(&self) -> ResourceIdent {
        match self {
            Resource::Agent(obj) => obj.resource_ident(),
            Resource::AgentSkill(obj) => obj.resource_ident(),
            Resource::Share(obj) => obj.resource_ident(),
            Resource::Credential(obj) => obj.resource_ident(),
            Resource::Catalog(obj) => obj.resource_ident(),
            Resource::Schema(obj) => obj.resource_ident(),
            Resource::Table(obj) => obj.resource_ident(),
            Resource::ExternalLocation(obj) => obj.resource_ident(),
            Resource::Recipient(obj) => obj.resource_ident(),
            Resource::Provider(obj) => obj.resource_ident(),
            Resource::Column(obj) => obj.resource_ident(),
            Resource::Volume(obj) => obj.resource_ident(),
            Resource::Function(obj) => obj.resource_ident(),
            Resource::TagPolicy(obj) => obj.resource_ident(),
            Resource::StagingTable(obj) => obj.resource_ident(),
            Resource::PolicyInfo(obj) => obj.resource_ident(),
            Resource::RegisteredModel(obj) => obj.resource_ident(),
            Resource::ModelVersion(obj) => obj.resource_ident(),
        }
    }
}

impl TryFrom<Resource> for Object {
    type Error = Error;

    fn try_from(resource: Resource) -> Result<Self, Self::Error> {
        let key = location_key(&resource);
        let mut object: Object = match resource {
            Resource::Agent(obj) => obj.try_into(),
            Resource::AgentSkill(obj) => obj.try_into(),
            Resource::Share(obj) => obj.try_into(),
            Resource::Credential(obj) => obj.try_into(),
            Resource::Catalog(obj) => obj.try_into(),
            Resource::Schema(obj) => obj.try_into(),
            Resource::Table(obj) => obj.try_into(),
            Resource::ExternalLocation(obj) => obj.try_into(),
            Resource::Recipient(obj) => obj.try_into(),
            Resource::Provider(obj) => obj.try_into(),
            Resource::Column(obj) => obj.try_into(),
            Resource::Volume(obj) => obj.try_into(),
            Resource::Function(obj) => obj.try_into(),
            Resource::TagPolicy(obj) => obj.try_into(),
            Resource::StagingTable(obj) => obj.try_into(),
            Resource::PolicyInfo(obj) => obj.try_into(),
            Resource::RegisteredModel(obj) => obj.try_into(),
            Resource::ModelVersion(obj) => obj.try_into(),
        }?;
        if let (Some(key), Some(serde_json::Value::Object(map))) = (key, &mut object.properties) {
            map.insert(LOCATION_KEY_PROPERTY.to_owned(), key.as_str().into());
        }
        Ok(object)
    }
}

/// The [`LocationKey`] stored with `resource`, for the resources looked up by location.
///
/// A missing or unparsable location gets no key, so it never matches a location query.
fn location_key(resource: &Resource) -> Option<LocationKey> {
    let location = match resource {
        Resource::Table(table) => table.storage_location.as_deref()?,
        Resource::Volume(volume) => &volume.storage_location,
        Resource::AgentSkill(skill) => &skill.storage_location,
        Resource::StagingTable(staging) => &staging.staging_location,
        _ => return None,
    };
    LocationKey::parse(location).ok()
}

impl TryFrom<Object> for Resource {
    type Error = Error;

    fn try_from(obj: Object) -> Result<Self, Self::Error> {
        match obj.label {
            ObjectLabel::Agent => Ok(Resource::Agent(obj.try_into()?)),
            ObjectLabel::AgentSkill => Ok(Resource::AgentSkill(obj.try_into()?)),
            ObjectLabel::Share => Ok(Resource::Share(obj.try_into()?)),
            ObjectLabel::Credential => Ok(Resource::Credential(obj.try_into()?)),
            ObjectLabel::Catalog => Ok(Resource::Catalog(obj.try_into()?)),
            ObjectLabel::Schema => Ok(Resource::Schema(obj.try_into()?)),
            ObjectLabel::Table => Ok(Resource::Table(obj.try_into()?)),
            ObjectLabel::ExternalLocation => Ok(Resource::ExternalLocation(obj.try_into()?)),
            ObjectLabel::Recipient => Ok(Resource::Recipient(obj.try_into()?)),
            ObjectLabel::Provider => Ok(Resource::Provider(obj.try_into()?)),
            ObjectLabel::Column => Ok(Resource::Column(obj.try_into()?)),
            ObjectLabel::Volume => Ok(Resource::Volume(obj.try_into()?)),
            ObjectLabel::Function => Ok(Resource::Function(obj.try_into()?)),
            ObjectLabel::TagPolicy => Ok(Resource::TagPolicy(obj.try_into()?)),
            ObjectLabel::StagingTable => Ok(Resource::StagingTable(obj.try_into()?)),
            ObjectLabel::PolicyInfo => Ok(Resource::PolicyInfo(obj.try_into()?)),
            ObjectLabel::RegisteredModel => Ok(Resource::RegisteredModel(obj.try_into()?)),
            ObjectLabel::ModelVersion => Ok(Resource::ModelVersion(obj.try_into()?)),
        }
    }
}

impl From<Table> for TableSummary {
    fn from(table: Table) -> Self {
        TableSummary {
            table_type: table.table_type,
            full_name: table.full_name,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::volumes::v1::Volume;

    fn stored_key(object: &Object) -> Option<&str> {
        object
            .properties
            .as_ref()?
            .get(LOCATION_KEY_PROPERTY)?
            .as_str()
    }

    #[test]
    fn stores_the_location_key_beside_the_payload() {
        let table = Table {
            name: "t".into(),
            catalog_name: "c".into(),
            schema_name: "s".into(),
            storage_location: Some("s3a://bucket/data/t".into()),
            ..Default::default()
        };
        let object = Object::try_from(Resource::Table(table.clone())).unwrap();
        assert_eq!(stored_key(&object), Some("s3://bucket/data/t/"));

        // The key stays out of the model read back.
        let Resource::Table(read) = Resource::try_from(object).unwrap() else {
            panic!("expected a table");
        };
        assert_eq!(read.storage_location, table.storage_location);
        assert_eq!(read.name, table.name);
    }

    #[test]
    fn skips_missing_and_unparsable_locations() {
        let table = Table {
            name: "t".into(),
            ..Default::default()
        };
        let object = Object::try_from(Resource::Table(table)).unwrap();
        assert_eq!(stored_key(&object), None);

        let volume = Volume {
            name: "v".into(),
            storage_location: "not a url".into(),
            ..Default::default()
        };
        let object = Object::try_from(Resource::Volume(volume)).unwrap();
        assert_eq!(stored_key(&object), None);
    }

    /// `location_key` is derived for exactly the labels that get an index.
    #[test]
    fn keyed_labels_match_the_derivation() {
        use crate::location_key::LOCATION_KEYED_LABELS;
        use crate::models::agent_skills::v0alpha1::AgentSkill;
        use crate::models::catalogs::v1::Catalog;
        use crate::models::staging_tables::v1::StagingTable;

        let location = "s3://bucket/x";
        let resources: Vec<Resource> = vec![
            Table {
                name: "t".into(),
                storage_location: Some(location.into()),
                ..Default::default()
            }
            .into(),
            Volume {
                name: "v".into(),
                storage_location: location.into(),
                ..Default::default()
            }
            .into(),
            AgentSkill {
                name: "a".into(),
                storage_location: location.into(),
                ..Default::default()
            }
            .into(),
            StagingTable {
                name: "s".into(),
                staging_location: location.into(),
                ..Default::default()
            }
            .into(),
            Catalog {
                name: "c".into(),
                storage_root: Some(location.into()),
                ..Default::default()
            }
            .into(),
        ];
        for resource in resources {
            let object = Object::try_from(resource).unwrap();
            assert_eq!(
                stored_key(&object).is_some(),
                LOCATION_KEYED_LABELS.contains(&object.label),
                "{:?}",
                object.label
            );
        }
    }
}
