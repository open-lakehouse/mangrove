// @generated
impl serde::Serialize for CreateEntityTagAssignmentRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tag_assignment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.CreateEntityTagAssignmentRequest", len)?;
        if let Some(v) = self.tag_assignment.as_ref() {
            struct_ser.serialize_field("tag_assignment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateEntityTagAssignmentRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag_assignment",
            "tagAssignment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TagAssignment,
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
                            "tagAssignment" | "tag_assignment" => Ok(GeneratedField::TagAssignment),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateEntityTagAssignmentRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.CreateEntityTagAssignmentRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateEntityTagAssignmentRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag_assignment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TagAssignment => {
                            if tag_assignment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagAssignment"));
                            }
                            tag_assignment__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CreateEntityTagAssignmentRequest {
                    tag_assignment: tag_assignment__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.CreateEntityTagAssignmentRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateTagPolicyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tag_policy.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.CreateTagPolicyRequest", len)?;
        if let Some(v) = self.tag_policy.as_ref() {
            struct_ser.serialize_field("tag_policy", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateTagPolicyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag_policy",
            "tagPolicy",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TagPolicy,
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
                            "tagPolicy" | "tag_policy" => Ok(GeneratedField::TagPolicy),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateTagPolicyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.CreateTagPolicyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateTagPolicyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag_policy__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TagPolicy => {
                            if tag_policy__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagPolicy"));
                            }
                            tag_policy__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CreateTagPolicyRequest {
                    tag_policy: tag_policy__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.CreateTagPolicyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteEntityTagAssignmentRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.entity_type.is_empty() {
            len += 1;
        }
        if !self.entity_name.is_empty() {
            len += 1;
        }
        if !self.tag_key.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.DeleteEntityTagAssignmentRequest", len)?;
        if !self.entity_type.is_empty() {
            struct_ser.serialize_field("entity_type", &self.entity_type)?;
        }
        if !self.entity_name.is_empty() {
            struct_ser.serialize_field("entity_name", &self.entity_name)?;
        }
        if !self.tag_key.is_empty() {
            struct_ser.serialize_field("tag_key", &self.tag_key)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteEntityTagAssignmentRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "entity_type",
            "entityType",
            "entity_name",
            "entityName",
            "tag_key",
            "tagKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EntityType,
            EntityName,
            TagKey,
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
                            "entityType" | "entity_type" => Ok(GeneratedField::EntityType),
                            "entityName" | "entity_name" => Ok(GeneratedField::EntityName),
                            "tagKey" | "tag_key" => Ok(GeneratedField::TagKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteEntityTagAssignmentRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.DeleteEntityTagAssignmentRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteEntityTagAssignmentRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut entity_type__ = None;
                let mut entity_name__ = None;
                let mut tag_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EntityType => {
                            if entity_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityType"));
                            }
                            entity_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EntityName => {
                            if entity_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityName"));
                            }
                            entity_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TagKey => {
                            if tag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagKey"));
                            }
                            tag_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DeleteEntityTagAssignmentRequest {
                    entity_type: entity_type__.unwrap_or_default(),
                    entity_name: entity_name__.unwrap_or_default(),
                    tag_key: tag_key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.DeleteEntityTagAssignmentRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteTagPolicyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tag_key.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.DeleteTagPolicyRequest", len)?;
        if !self.tag_key.is_empty() {
            struct_ser.serialize_field("tag_key", &self.tag_key)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteTagPolicyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag_key",
            "tagKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TagKey,
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
                            "tagKey" | "tag_key" => Ok(GeneratedField::TagKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteTagPolicyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.DeleteTagPolicyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteTagPolicyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TagKey => {
                            if tag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagKey"));
                            }
                            tag_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DeleteTagPolicyRequest {
                    tag_key: tag_key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.DeleteTagPolicyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EntityTagAssignment {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.entity_type.is_empty() {
            len += 1;
        }
        if !self.entity_name.is_empty() {
            len += 1;
        }
        if !self.tag_key.is_empty() {
            len += 1;
        }
        if self.tag_value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.EntityTagAssignment", len)?;
        if !self.entity_type.is_empty() {
            struct_ser.serialize_field("entity_type", &self.entity_type)?;
        }
        if !self.entity_name.is_empty() {
            struct_ser.serialize_field("entity_name", &self.entity_name)?;
        }
        if !self.tag_key.is_empty() {
            struct_ser.serialize_field("tag_key", &self.tag_key)?;
        }
        if let Some(v) = self.tag_value.as_ref() {
            struct_ser.serialize_field("tag_value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EntityTagAssignment {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "entity_type",
            "entityType",
            "entity_name",
            "entityName",
            "tag_key",
            "tagKey",
            "tag_value",
            "tagValue",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EntityType,
            EntityName,
            TagKey,
            TagValue,
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
                            "entityType" | "entity_type" => Ok(GeneratedField::EntityType),
                            "entityName" | "entity_name" => Ok(GeneratedField::EntityName),
                            "tagKey" | "tag_key" => Ok(GeneratedField::TagKey),
                            "tagValue" | "tag_value" => Ok(GeneratedField::TagValue),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EntityTagAssignment;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.EntityTagAssignment")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EntityTagAssignment, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut entity_type__ = None;
                let mut entity_name__ = None;
                let mut tag_key__ = None;
                let mut tag_value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EntityType => {
                            if entity_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityType"));
                            }
                            entity_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EntityName => {
                            if entity_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityName"));
                            }
                            entity_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TagKey => {
                            if tag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagKey"));
                            }
                            tag_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TagValue => {
                            if tag_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagValue"));
                            }
                            tag_value__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(EntityTagAssignment {
                    entity_type: entity_type__.unwrap_or_default(),
                    entity_name: entity_name__.unwrap_or_default(),
                    tag_key: tag_key__.unwrap_or_default(),
                    tag_value: tag_value__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.EntityTagAssignment", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetEntityTagAssignmentRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.entity_type.is_empty() {
            len += 1;
        }
        if !self.entity_name.is_empty() {
            len += 1;
        }
        if !self.tag_key.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.GetEntityTagAssignmentRequest", len)?;
        if !self.entity_type.is_empty() {
            struct_ser.serialize_field("entity_type", &self.entity_type)?;
        }
        if !self.entity_name.is_empty() {
            struct_ser.serialize_field("entity_name", &self.entity_name)?;
        }
        if !self.tag_key.is_empty() {
            struct_ser.serialize_field("tag_key", &self.tag_key)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetEntityTagAssignmentRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "entity_type",
            "entityType",
            "entity_name",
            "entityName",
            "tag_key",
            "tagKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EntityType,
            EntityName,
            TagKey,
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
                            "entityType" | "entity_type" => Ok(GeneratedField::EntityType),
                            "entityName" | "entity_name" => Ok(GeneratedField::EntityName),
                            "tagKey" | "tag_key" => Ok(GeneratedField::TagKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetEntityTagAssignmentRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.GetEntityTagAssignmentRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetEntityTagAssignmentRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut entity_type__ = None;
                let mut entity_name__ = None;
                let mut tag_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EntityType => {
                            if entity_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityType"));
                            }
                            entity_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EntityName => {
                            if entity_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityName"));
                            }
                            entity_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TagKey => {
                            if tag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagKey"));
                            }
                            tag_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GetEntityTagAssignmentRequest {
                    entity_type: entity_type__.unwrap_or_default(),
                    entity_name: entity_name__.unwrap_or_default(),
                    tag_key: tag_key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.GetEntityTagAssignmentRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetTagPolicyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tag_key.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.GetTagPolicyRequest", len)?;
        if !self.tag_key.is_empty() {
            struct_ser.serialize_field("tag_key", &self.tag_key)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetTagPolicyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag_key",
            "tagKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TagKey,
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
                            "tagKey" | "tag_key" => Ok(GeneratedField::TagKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetTagPolicyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.GetTagPolicyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetTagPolicyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TagKey => {
                            if tag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagKey"));
                            }
                            tag_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GetTagPolicyRequest {
                    tag_key: tag_key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.GetTagPolicyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListEntityTagAssignmentsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.entity_type.is_empty() {
            len += 1;
        }
        if !self.entity_name.is_empty() {
            len += 1;
        }
        if self.max_results.is_some() {
            len += 1;
        }
        if self.page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.ListEntityTagAssignmentsRequest", len)?;
        if !self.entity_type.is_empty() {
            struct_ser.serialize_field("entity_type", &self.entity_type)?;
        }
        if !self.entity_name.is_empty() {
            struct_ser.serialize_field("entity_name", &self.entity_name)?;
        }
        if let Some(v) = self.max_results.as_ref() {
            struct_ser.serialize_field("max_results", v)?;
        }
        if let Some(v) = self.page_token.as_ref() {
            struct_ser.serialize_field("page_token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListEntityTagAssignmentsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "entity_type",
            "entityType",
            "entity_name",
            "entityName",
            "max_results",
            "maxResults",
            "page_token",
            "pageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EntityType,
            EntityName,
            MaxResults,
            PageToken,
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
                            "entityType" | "entity_type" => Ok(GeneratedField::EntityType),
                            "entityName" | "entity_name" => Ok(GeneratedField::EntityName),
                            "maxResults" | "max_results" => Ok(GeneratedField::MaxResults),
                            "pageToken" | "page_token" => Ok(GeneratedField::PageToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListEntityTagAssignmentsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.ListEntityTagAssignmentsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListEntityTagAssignmentsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut entity_type__ = None;
                let mut entity_name__ = None;
                let mut max_results__ = None;
                let mut page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EntityType => {
                            if entity_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityType"));
                            }
                            entity_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EntityName => {
                            if entity_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityName"));
                            }
                            entity_name__ = Some(map_.next_value()?);
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ListEntityTagAssignmentsRequest {
                    entity_type: entity_type__.unwrap_or_default(),
                    entity_name: entity_name__.unwrap_or_default(),
                    max_results: max_results__,
                    page_token: page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.ListEntityTagAssignmentsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListEntityTagAssignmentsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tag_assignments.is_empty() {
            len += 1;
        }
        if self.next_page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.ListEntityTagAssignmentsResponse", len)?;
        if !self.tag_assignments.is_empty() {
            struct_ser.serialize_field("tag_assignments", &self.tag_assignments)?;
        }
        if let Some(v) = self.next_page_token.as_ref() {
            struct_ser.serialize_field("next_page_token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListEntityTagAssignmentsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag_assignments",
            "tagAssignments",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TagAssignments,
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
                            "tagAssignments" | "tag_assignments" => Ok(GeneratedField::TagAssignments),
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
            type Value = ListEntityTagAssignmentsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.ListEntityTagAssignmentsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListEntityTagAssignmentsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag_assignments__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TagAssignments => {
                            if tag_assignments__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagAssignments"));
                            }
                            tag_assignments__ = Some(map_.next_value()?);
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
                Ok(ListEntityTagAssignmentsResponse {
                    tag_assignments: tag_assignments__.unwrap_or_default(),
                    next_page_token: next_page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.ListEntityTagAssignmentsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListTagPoliciesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.max_results.is_some() {
            len += 1;
        }
        if self.page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.ListTagPoliciesRequest", len)?;
        if let Some(v) = self.max_results.as_ref() {
            struct_ser.serialize_field("max_results", v)?;
        }
        if let Some(v) = self.page_token.as_ref() {
            struct_ser.serialize_field("page_token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListTagPoliciesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "max_results",
            "maxResults",
            "page_token",
            "pageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MaxResults,
            PageToken,
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
                            "maxResults" | "max_results" => Ok(GeneratedField::MaxResults),
                            "pageToken" | "page_token" => Ok(GeneratedField::PageToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ListTagPoliciesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.ListTagPoliciesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListTagPoliciesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut max_results__ = None;
                let mut page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(ListTagPoliciesRequest {
                    max_results: max_results__,
                    page_token: page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.ListTagPoliciesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListTagPoliciesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tag_policies.is_empty() {
            len += 1;
        }
        if self.next_page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.ListTagPoliciesResponse", len)?;
        if !self.tag_policies.is_empty() {
            struct_ser.serialize_field("tag_policies", &self.tag_policies)?;
        }
        if let Some(v) = self.next_page_token.as_ref() {
            struct_ser.serialize_field("next_page_token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListTagPoliciesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag_policies",
            "tagPolicies",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TagPolicies,
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
                            "tagPolicies" | "tag_policies" => Ok(GeneratedField::TagPolicies),
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
            type Value = ListTagPoliciesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.ListTagPoliciesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListTagPoliciesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag_policies__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TagPolicies => {
                            if tag_policies__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagPolicies"));
                            }
                            tag_policies__ = Some(map_.next_value()?);
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
                Ok(ListTagPoliciesResponse {
                    tag_policies: tag_policies__.unwrap_or_default(),
                    next_page_token: next_page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.ListTagPoliciesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TagPolicy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tag_key.is_empty() {
            len += 1;
        }
        if self.description.is_some() {
            len += 1;
        }
        if !self.values.is_empty() {
            len += 1;
        }
        if self.id.is_some() {
            len += 1;
        }
        if self.created_at.is_some() {
            len += 1;
        }
        if self.updated_at.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.TagPolicy", len)?;
        if !self.tag_key.is_empty() {
            struct_ser.serialize_field("tag_key", &self.tag_key)?;
        }
        if let Some(v) = self.description.as_ref() {
            struct_ser.serialize_field("description", v)?;
        }
        if !self.values.is_empty() {
            struct_ser.serialize_field("values", &self.values)?;
        }
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("created_at", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.updated_at.as_ref() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("updated_at", ToString::to_string(&v).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TagPolicy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag_key",
            "tagKey",
            "description",
            "values",
            "id",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TagKey,
            Description,
            Values,
            Id,
            CreatedAt,
            UpdatedAt,
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
                            "tagKey" | "tag_key" => Ok(GeneratedField::TagKey),
                            "description" => Ok(GeneratedField::Description),
                            "values" => Ok(GeneratedField::Values),
                            "id" => Ok(GeneratedField::Id),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TagPolicy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.TagPolicy")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TagPolicy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag_key__ = None;
                let mut description__ = None;
                let mut values__ = None;
                let mut id__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TagKey => {
                            if tag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagKey"));
                            }
                            tag_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = map_.next_value()?;
                        }
                        GeneratedField::Values => {
                            if values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            values__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TagPolicy {
                    tag_key: tag_key__.unwrap_or_default(),
                    description: description__,
                    values: values__.unwrap_or_default(),
                    id: id__,
                    created_at: created_at__,
                    updated_at: updated_at__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.TagPolicy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateEntityTagAssignmentRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.entity_type.is_empty() {
            len += 1;
        }
        if !self.entity_name.is_empty() {
            len += 1;
        }
        if !self.tag_key.is_empty() {
            len += 1;
        }
        if self.tag_assignment.is_some() {
            len += 1;
        }
        if self.update_mask.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.UpdateEntityTagAssignmentRequest", len)?;
        if !self.entity_type.is_empty() {
            struct_ser.serialize_field("entity_type", &self.entity_type)?;
        }
        if !self.entity_name.is_empty() {
            struct_ser.serialize_field("entity_name", &self.entity_name)?;
        }
        if !self.tag_key.is_empty() {
            struct_ser.serialize_field("tag_key", &self.tag_key)?;
        }
        if let Some(v) = self.tag_assignment.as_ref() {
            struct_ser.serialize_field("tag_assignment", v)?;
        }
        if let Some(v) = self.update_mask.as_ref() {
            struct_ser.serialize_field("update_mask", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateEntityTagAssignmentRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "entity_type",
            "entityType",
            "entity_name",
            "entityName",
            "tag_key",
            "tagKey",
            "tag_assignment",
            "tagAssignment",
            "update_mask",
            "updateMask",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            EntityType,
            EntityName,
            TagKey,
            TagAssignment,
            UpdateMask,
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
                            "entityType" | "entity_type" => Ok(GeneratedField::EntityType),
                            "entityName" | "entity_name" => Ok(GeneratedField::EntityName),
                            "tagKey" | "tag_key" => Ok(GeneratedField::TagKey),
                            "tagAssignment" | "tag_assignment" => Ok(GeneratedField::TagAssignment),
                            "updateMask" | "update_mask" => Ok(GeneratedField::UpdateMask),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateEntityTagAssignmentRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.UpdateEntityTagAssignmentRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateEntityTagAssignmentRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut entity_type__ = None;
                let mut entity_name__ = None;
                let mut tag_key__ = None;
                let mut tag_assignment__ = None;
                let mut update_mask__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::EntityType => {
                            if entity_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityType"));
                            }
                            entity_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::EntityName => {
                            if entity_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("entityName"));
                            }
                            entity_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TagKey => {
                            if tag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagKey"));
                            }
                            tag_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TagAssignment => {
                            if tag_assignment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagAssignment"));
                            }
                            tag_assignment__ = map_.next_value()?;
                        }
                        GeneratedField::UpdateMask => {
                            if update_mask__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateMask"));
                            }
                            update_mask__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdateEntityTagAssignmentRequest {
                    entity_type: entity_type__.unwrap_or_default(),
                    entity_name: entity_name__.unwrap_or_default(),
                    tag_key: tag_key__.unwrap_or_default(),
                    tag_assignment: tag_assignment__,
                    update_mask: update_mask__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.UpdateEntityTagAssignmentRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateTagPolicyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.tag_key.is_empty() {
            len += 1;
        }
        if self.tag_policy.is_some() {
            len += 1;
        }
        if self.update_mask.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.UpdateTagPolicyRequest", len)?;
        if !self.tag_key.is_empty() {
            struct_ser.serialize_field("tag_key", &self.tag_key)?;
        }
        if let Some(v) = self.tag_policy.as_ref() {
            struct_ser.serialize_field("tag_policy", v)?;
        }
        if let Some(v) = self.update_mask.as_ref() {
            struct_ser.serialize_field("update_mask", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateTagPolicyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tag_key",
            "tagKey",
            "tag_policy",
            "tagPolicy",
            "update_mask",
            "updateMask",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TagKey,
            TagPolicy,
            UpdateMask,
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
                            "tagKey" | "tag_key" => Ok(GeneratedField::TagKey),
                            "tagPolicy" | "tag_policy" => Ok(GeneratedField::TagPolicy),
                            "updateMask" | "update_mask" => Ok(GeneratedField::UpdateMask),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateTagPolicyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.UpdateTagPolicyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateTagPolicyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tag_key__ = None;
                let mut tag_policy__ = None;
                let mut update_mask__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TagKey => {
                            if tag_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagKey"));
                            }
                            tag_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TagPolicy => {
                            if tag_policy__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tagPolicy"));
                            }
                            tag_policy__ = map_.next_value()?;
                        }
                        GeneratedField::UpdateMask => {
                            if update_mask__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updateMask"));
                            }
                            update_mask__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdateTagPolicyRequest {
                    tag_key: tag_key__.unwrap_or_default(),
                    tag_policy: tag_policy__,
                    update_mask: update_mask__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.UpdateTagPolicyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Value {
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
        let mut struct_ser = serializer.serialize_struct("unitycatalog.tags.v1.Value", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Value {
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
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.tags.v1.Value")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Value, V::Error>
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
                Ok(Value {
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.tags.v1.Value", FIELDS, GeneratedVisitor)
    }
}
