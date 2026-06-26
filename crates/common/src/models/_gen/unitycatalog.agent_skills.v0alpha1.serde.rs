// @generated
impl serde::Serialize for AgentSkill {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.catalog_name.is_empty() {
            len += 1;
        }
        if !self.schema_name.is_empty() {
            len += 1;
        }
        if !self.full_name.is_empty() {
            len += 1;
        }
        if !self.storage_location.is_empty() {
            len += 1;
        }
        if !self.agent_skill_id.is_empty() {
            len += 1;
        }
        if self.agent_skill_type != 0 {
            len += 1;
        }
        if self.description.is_some() {
            len += 1;
        }
        if self.license.is_some() {
            len += 1;
        }
        if !self.allowed_tools.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if self.owner.is_some() {
            len += 1;
        }
        if self.comment.is_some() {
            len += 1;
        }
        if self.created_at.is_some() {
            len += 1;
        }
        if self.created_by.is_some() {
            len += 1;
        }
        if self.updated_at.is_some() {
            len += 1;
        }
        if self.updated_by.is_some() {
            len += 1;
        }
        if self.metastore_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.agent_skills.v0alpha1.AgentSkill", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.catalog_name.is_empty() {
            struct_ser.serialize_field("catalog_name", &self.catalog_name)?;
        }
        if !self.schema_name.is_empty() {
            struct_ser.serialize_field("schema_name", &self.schema_name)?;
        }
        if !self.full_name.is_empty() {
            struct_ser.serialize_field("full_name", &self.full_name)?;
        }
        if !self.storage_location.is_empty() {
            struct_ser.serialize_field("storage_location", &self.storage_location)?;
        }
        if !self.agent_skill_id.is_empty() {
            struct_ser.serialize_field("agent_skill_id", &self.agent_skill_id)?;
        }
        if self.agent_skill_type != 0 {
            let v = AgentSkillType::try_from(self.agent_skill_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.agent_skill_type)))?;
            struct_ser.serialize_field("agent_skill_type", &v)?;
        }
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if let Some(v) = self.license.as_ref() {
            struct_ser.serialize_field("license", v)?;
        }
        if !self.allowed_tools.is_empty() {
            struct_ser.serialize_field("allowed_tools", &self.allowed_tools)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if let Some(v) = self.owner.as_ref() {
            struct_ser.serialize_field("owner", v)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("created_at", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.created_by.as_ref() {
            struct_ser.serialize_field("created_by", v)?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("updated_at", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.updated_by.as_ref() {
            struct_ser.serialize_field("updated_by", v)?;
        }
        if let Some(v) = self.metastore_id.as_ref() {
            struct_ser.serialize_field("metastore_id", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AgentSkill {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "catalog_name",
            "catalogName",
            "schema_name",
            "schemaName",
            "full_name",
            "fullName",
            "storage_location",
            "storageLocation",
            "agent_skill_id",
            "agentSkillId",
            "agent_skill_type",
            "agentSkillType",
            "description",
            "license",
            "allowed_tools",
            "allowedTools",
            "metadata",
            "owner",
            "comment",
            "created_at",
            "createdAt",
            "created_by",
            "createdBy",
            "updated_at",
            "updatedAt",
            "updated_by",
            "updatedBy",
            "metastore_id",
            "metastoreId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            CatalogName,
            SchemaName,
            FullName,
            StorageLocation,
            AgentSkillId,
            AgentSkillType,
            Description,
            License,
            AllowedTools,
            Metadata,
            Owner,
            Comment,
            CreatedAt,
            CreatedBy,
            UpdatedAt,
            UpdatedBy,
            MetastoreId,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "catalogName" | "catalog_name" => Ok(GeneratedField::CatalogName),
                            "schemaName" | "schema_name" => Ok(GeneratedField::SchemaName),
                            "fullName" | "full_name" => Ok(GeneratedField::FullName),
                            "storageLocation" | "storage_location" => Ok(GeneratedField::StorageLocation),
                            "agentSkillId" | "agent_skill_id" => Ok(GeneratedField::AgentSkillId),
                            "agentSkillType" | "agent_skill_type" => Ok(GeneratedField::AgentSkillType),
                            "description" => Ok(GeneratedField::Description),
                            "license" => Ok(GeneratedField::License),
                            "allowedTools" | "allowed_tools" => Ok(GeneratedField::AllowedTools),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "owner" => Ok(GeneratedField::Owner),
                            "comment" => Ok(GeneratedField::Comment),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "createdBy" | "created_by" => Ok(GeneratedField::CreatedBy),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            "updatedBy" | "updated_by" => Ok(GeneratedField::UpdatedBy),
                            "metastoreId" | "metastore_id" => Ok(GeneratedField::MetastoreId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AgentSkill;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.agent_skills.v0alpha1.AgentSkill")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AgentSkill, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut catalog_name__ = None;
                let mut schema_name__ = None;
                let mut full_name__ = None;
                let mut storage_location__ = None;
                let mut agent_skill_id__ = None;
                let mut agent_skill_type__ = None;
                let mut description__ = None;
                let mut license__ = None;
                let mut allowed_tools__ = None;
                let mut metadata__ = None;
                let mut owner__ = None;
                let mut comment__ = None;
                let mut created_at__ = None;
                let mut created_by__ = None;
                let mut updated_at__ = None;
                let mut updated_by__ = None;
                let mut metastore_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CatalogName => {
                            if catalog_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("catalogName"));
                            }
                            catalog_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SchemaName => {
                            if schema_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaName"));
                            }
                            schema_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullName => {
                            if full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullName"));
                            }
                            full_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::StorageLocation => {
                            if storage_location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("storageLocation"));
                            }
                            storage_location__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AgentSkillId => {
                            if agent_skill_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("agentSkillId"));
                            }
                            agent_skill_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AgentSkillType => {
                            if agent_skill_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("agentSkillType"));
                            }
                            agent_skill_type__ = Some(map_.next_value::<AgentSkillType>()? as i32);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::License => {
                            if license__.is_some() {
                                return Err(serde::de::Error::duplicate_field("license"));
                            }
                            license__ = map_.next_value()?;
                        }
                        GeneratedField::AllowedTools => {
                            if allowed_tools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowedTools"));
                            }
                            allowed_tools__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Owner => {
                            if owner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("owner"));
                            }
                            owner__ = map_.next_value()?;
                        }
                        GeneratedField::Comment => {
                            if comment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("comment"));
                            }
                            comment__ = map_.next_value()?;
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::CreatedBy => {
                            if created_by__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdBy"));
                            }
                            created_by__ = map_.next_value()?;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::UpdatedBy => {
                            if updated_by__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedBy"));
                            }
                            updated_by__ = map_.next_value()?;
                        }
                        GeneratedField::MetastoreId => {
                            if metastore_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metastoreId"));
                            }
                            metastore_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AgentSkill {
                    name: name__.unwrap_or_default(),
                    catalog_name: catalog_name__.unwrap_or_default(),
                    schema_name: schema_name__.unwrap_or_default(),
                    full_name: full_name__.unwrap_or_default(),
                    storage_location: storage_location__.unwrap_or_default(),
                    agent_skill_id: agent_skill_id__.unwrap_or_default(),
                    agent_skill_type: agent_skill_type__.unwrap_or_default(),
                    description: description__,
                    license: license__,
                    allowed_tools: allowed_tools__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    owner: owner__,
                    comment: comment__,
                    created_at: created_at__,
                    created_by: created_by__,
                    updated_at: updated_at__,
                    updated_by: updated_by__,
                    metastore_id: metastore_id__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.agent_skills.v0alpha1.AgentSkill", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AgentSkillType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "AGENT_SKILL_TYPE_UNSPECIFIED",
            Self::External => "EXTERNAL",
            Self::Managed => "MANAGED",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for AgentSkillType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "AGENT_SKILL_TYPE_UNSPECIFIED",
            "EXTERNAL",
            "MANAGED",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AgentSkillType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                i32::try_from(v)
                    .ok()
                    .and_then(|x| x.try_into().ok())
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "AGENT_SKILL_TYPE_UNSPECIFIED" => Ok(AgentSkillType::Unspecified),
                    "EXTERNAL" => Ok(AgentSkillType::External),
                    "MANAGED" => Ok(AgentSkillType::Managed),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for CreateAgentSkillRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.catalog_name.is_empty() {
            len += 1;
        }
        if !self.schema_name.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if self.agent_skill_type != 0 {
            len += 1;
        }
        if self.storage_location.is_some() {
            len += 1;
        }
        if self.description.is_some() {
            len += 1;
        }
        if self.license.is_some() {
            len += 1;
        }
        if !self.allowed_tools.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if self.comment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.agent_skills.v0alpha1.CreateAgentSkillRequest", len)?;
        if !self.catalog_name.is_empty() {
            struct_ser.serialize_field("catalog_name", &self.catalog_name)?;
        }
        if !self.schema_name.is_empty() {
            struct_ser.serialize_field("schema_name", &self.schema_name)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.agent_skill_type != 0 {
            let v = AgentSkillType::try_from(self.agent_skill_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.agent_skill_type)))?;
            struct_ser.serialize_field("agent_skill_type", &v)?;
        }
        if let Some(v) = self.storage_location.as_ref() {
            struct_ser.serialize_field("storage_location", v)?;
        }
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if let Some(v) = self.license.as_ref() {
            struct_ser.serialize_field("license", v)?;
        }
        if !self.allowed_tools.is_empty() {
            struct_ser.serialize_field("allowed_tools", &self.allowed_tools)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateAgentSkillRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "catalog_name",
            "catalogName",
            "schema_name",
            "schemaName",
            "name",
            "agent_skill_type",
            "agentSkillType",
            "storage_location",
            "storageLocation",
            "description",
            "license",
            "allowed_tools",
            "allowedTools",
            "metadata",
            "comment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CatalogName,
            SchemaName,
            Name,
            AgentSkillType,
            StorageLocation,
            Description,
            License,
            AllowedTools,
            Metadata,
            Comment,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "catalogName" | "catalog_name" => Ok(GeneratedField::CatalogName),
                            "schemaName" | "schema_name" => Ok(GeneratedField::SchemaName),
                            "name" => Ok(GeneratedField::Name),
                            "agentSkillType" | "agent_skill_type" => Ok(GeneratedField::AgentSkillType),
                            "storageLocation" | "storage_location" => Ok(GeneratedField::StorageLocation),
                            "description" => Ok(GeneratedField::Description),
                            "license" => Ok(GeneratedField::License),
                            "allowedTools" | "allowed_tools" => Ok(GeneratedField::AllowedTools),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "comment" => Ok(GeneratedField::Comment),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateAgentSkillRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.agent_skills.v0alpha1.CreateAgentSkillRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateAgentSkillRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut catalog_name__ = None;
                let mut schema_name__ = None;
                let mut name__ = None;
                let mut agent_skill_type__ = None;
                let mut storage_location__ = None;
                let mut description__ = None;
                let mut license__ = None;
                let mut allowed_tools__ = None;
                let mut metadata__ = None;
                let mut comment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CatalogName => {
                            if catalog_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("catalogName"));
                            }
                            catalog_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SchemaName => {
                            if schema_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaName"));
                            }
                            schema_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AgentSkillType => {
                            if agent_skill_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("agentSkillType"));
                            }
                            agent_skill_type__ = Some(map_.next_value::<AgentSkillType>()? as i32);
                        }
                        GeneratedField::StorageLocation => {
                            if storage_location__.is_some() {
                                return Err(serde::de::Error::duplicate_field("storageLocation"));
                            }
                            storage_location__ = map_.next_value()?;
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::License => {
                            if license__.is_some() {
                                return Err(serde::de::Error::duplicate_field("license"));
                            }
                            license__ = map_.next_value()?;
                        }
                        GeneratedField::AllowedTools => {
                            if allowed_tools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowedTools"));
                            }
                            allowed_tools__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Comment => {
                            if comment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("comment"));
                            }
                            comment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CreateAgentSkillRequest {
                    catalog_name: catalog_name__.unwrap_or_default(),
                    schema_name: schema_name__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    agent_skill_type: agent_skill_type__.unwrap_or_default(),
                    storage_location: storage_location__,
                    description: description__,
                    license: license__,
                    allowed_tools: allowed_tools__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    comment: comment__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.agent_skills.v0alpha1.CreateAgentSkillRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteAgentSkillRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.agent_skills.v0alpha1.DeleteAgentSkillRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteAgentSkillRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteAgentSkillRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.agent_skills.v0alpha1.DeleteAgentSkillRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteAgentSkillRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DeleteAgentSkillRequest {
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.agent_skills.v0alpha1.DeleteAgentSkillRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetAgentSkillRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if self.include_browse.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.agent_skills.v0alpha1.GetAgentSkillRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.include_browse.as_ref() {
            struct_ser.serialize_field("include_browse", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetAgentSkillRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "include_browse",
            "includeBrowse",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            IncludeBrowse,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "includeBrowse" | "include_browse" => Ok(GeneratedField::IncludeBrowse),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetAgentSkillRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.agent_skills.v0alpha1.GetAgentSkillRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetAgentSkillRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut include_browse__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IncludeBrowse => {
                            if include_browse__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeBrowse"));
                            }
                            include_browse__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GetAgentSkillRequest {
                    name: name__.unwrap_or_default(),
                    include_browse: include_browse__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.agent_skills.v0alpha1.GetAgentSkillRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListAgentSkillsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.catalog_name.is_empty() {
            len += 1;
        }
        if !self.schema_name.is_empty() {
            len += 1;
        }
        if self.max_results.is_some() {
            len += 1;
        }
        if self.page_token.is_some() {
            len += 1;
        }
        if self.include_browse.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.agent_skills.v0alpha1.ListAgentSkillsRequest", len)?;
        if !self.catalog_name.is_empty() {
            struct_ser.serialize_field("catalog_name", &self.catalog_name)?;
        }
        if !self.schema_name.is_empty() {
            struct_ser.serialize_field("schema_name", &self.schema_name)?;
        }
        if let Some(v) = self.max_results.as_ref() {
            struct_ser.serialize_field("max_results", v)?;
        }
        if let Some(v) = self.page_token.as_ref() {
            struct_ser.serialize_field("page_token", v)?;
        }
        if let Some(v) = self.include_browse.as_ref() {
            struct_ser.serialize_field("include_browse", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListAgentSkillsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "catalog_name",
            "catalogName",
            "schema_name",
            "schemaName",
            "max_results",
            "maxResults",
            "page_token",
            "pageToken",
            "include_browse",
            "includeBrowse",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CatalogName,
            SchemaName,
            MaxResults,
            PageToken,
            IncludeBrowse,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "catalogName" | "catalog_name" => Ok(GeneratedField::CatalogName),
                            "schemaName" | "schema_name" => Ok(GeneratedField::SchemaName),
                            "maxResults" | "max_results" => Ok(GeneratedField::MaxResults),
                            "pageToken" | "page_token" => Ok(GeneratedField::PageToken),
                            "includeBrowse" | "include_browse" => Ok(GeneratedField::IncludeBrowse),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListAgentSkillsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.agent_skills.v0alpha1.ListAgentSkillsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListAgentSkillsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut catalog_name__ = None;
                let mut schema_name__ = None;
                let mut max_results__ = None;
                let mut page_token__ = None;
                let mut include_browse__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CatalogName => {
                            if catalog_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("catalogName"));
                            }
                            catalog_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SchemaName => {
                            if schema_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaName"));
                            }
                            schema_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::MaxResults => {
                            if max_results__.is_some() {
                                return Err(serde::de::Error::duplicate_field("maxResults"));
                            }
                            max_results__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::PageToken => {
                            if page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pageToken"));
                            }
                            page_token__ = map_.next_value()?;
                        }
                        GeneratedField::IncludeBrowse => {
                            if include_browse__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeBrowse"));
                            }
                            include_browse__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ListAgentSkillsRequest {
                    catalog_name: catalog_name__.unwrap_or_default(),
                    schema_name: schema_name__.unwrap_or_default(),
                    max_results: max_results__,
                    page_token: page_token__,
                    include_browse: include_browse__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.agent_skills.v0alpha1.ListAgentSkillsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListAgentSkillsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.agent_skills.is_empty() {
            len += 1;
        }
        if self.next_page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.agent_skills.v0alpha1.ListAgentSkillsResponse", len)?;
        if !self.agent_skills.is_empty() {
            struct_ser.serialize_field("agent_skills", &self.agent_skills)?;
        }
        if let Some(v) = self.next_page_token.as_ref() {
            struct_ser.serialize_field("next_page_token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListAgentSkillsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "agent_skills",
            "agentSkills",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AgentSkills,
            NextPageToken,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "agentSkills" | "agent_skills" => Ok(GeneratedField::AgentSkills),
                            "nextPageToken" | "next_page_token" => Ok(GeneratedField::NextPageToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListAgentSkillsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.agent_skills.v0alpha1.ListAgentSkillsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListAgentSkillsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut agent_skills__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AgentSkills => {
                            if agent_skills__.is_some() {
                                return Err(serde::de::Error::duplicate_field("agentSkills"));
                            }
                            agent_skills__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NextPageToken => {
                            if next_page_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("nextPageToken"));
                            }
                            next_page_token__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ListAgentSkillsResponse {
                    agent_skills: agent_skills__.unwrap_or_default(),
                    next_page_token: next_page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.agent_skills.v0alpha1.ListAgentSkillsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateAgentSkillRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if self.new_name.is_some() {
            len += 1;
        }
        if self.description.is_some() {
            len += 1;
        }
        if !self.allowed_tools.is_empty() {
            len += 1;
        }
        if self.comment.is_some() {
            len += 1;
        }
        if self.owner.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.agent_skills.v0alpha1.UpdateAgentSkillRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.new_name.as_ref() {
            struct_ser.serialize_field("new_name", v)?;
        }
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if !self.allowed_tools.is_empty() {
            struct_ser.serialize_field("allowed_tools", &self.allowed_tools)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
        }
        if let Some(v) = self.owner.as_ref() {
            struct_ser.serialize_field("owner", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateAgentSkillRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "new_name",
            "newName",
            "description",
            "allowed_tools",
            "allowedTools",
            "comment",
            "owner",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            NewName,
            Description,
            AllowedTools,
            Comment,
            Owner,
            __SkipField__,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "newName" | "new_name" => Ok(GeneratedField::NewName),
                            "description" => Ok(GeneratedField::Description),
                            "allowedTools" | "allowed_tools" => Ok(GeneratedField::AllowedTools),
                            "comment" => Ok(GeneratedField::Comment),
                            "owner" => Ok(GeneratedField::Owner),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateAgentSkillRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.agent_skills.v0alpha1.UpdateAgentSkillRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateAgentSkillRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut new_name__ = None;
                let mut description__ = None;
                let mut allowed_tools__ = None;
                let mut comment__ = None;
                let mut owner__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::NewName => {
                            if new_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newName"));
                            }
                            new_name__ = map_.next_value()?;
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::AllowedTools => {
                            if allowed_tools__.is_some() {
                                return Err(serde::de::Error::duplicate_field("allowedTools"));
                            }
                            allowed_tools__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Comment => {
                            if comment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("comment"));
                            }
                            comment__ = map_.next_value()?;
                        }
                        GeneratedField::Owner => {
                            if owner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("owner"));
                            }
                            owner__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdateAgentSkillRequest {
                    name: name__.unwrap_or_default(),
                    new_name: new_name__,
                    description: description__,
                    allowed_tools: allowed_tools__.unwrap_or_default(),
                    comment: comment__,
                    owner: owner__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.agent_skills.v0alpha1.UpdateAgentSkillRequest", FIELDS, GeneratedVisitor)
    }
}
