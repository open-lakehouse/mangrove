// @generated
impl serde::Serialize for CommitInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.version != 0 {
            len += 1;
        }
        if self.timestamp != 0 {
            len += 1;
        }
        if !self.file_name.is_empty() {
            len += 1;
        }
        if self.file_size != 0 {
            len += 1;
        }
        if self.file_modification_timestamp != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.delta_commits.v1.CommitInfo", len)?;
        if self.version != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("version", ToString::to_string(&self.version).as_str())?;
        }
        if self.timestamp != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("timestamp", ToString::to_string(&self.timestamp).as_str())?;
        }
        if !self.file_name.is_empty() {
            struct_ser.serialize_field("file_name", &self.file_name)?;
        }
        if self.file_size != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("file_size", ToString::to_string(&self.file_size).as_str())?;
        }
        if self.file_modification_timestamp != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("file_modification_timestamp", ToString::to_string(&self.file_modification_timestamp).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CommitInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "version",
            "timestamp",
            "file_name",
            "fileName",
            "file_size",
            "fileSize",
            "file_modification_timestamp",
            "fileModificationTimestamp",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Version,
            Timestamp,
            FileName,
            FileSize,
            FileModificationTimestamp,
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
                            "version" => Ok(GeneratedField::Version),
                            "timestamp" => Ok(GeneratedField::Timestamp),
                            "fileName" | "file_name" => Ok(GeneratedField::FileName),
                            "fileSize" | "file_size" => Ok(GeneratedField::FileSize),
                            "fileModificationTimestamp" | "file_modification_timestamp" => Ok(GeneratedField::FileModificationTimestamp),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CommitInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.delta_commits.v1.CommitInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CommitInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut version__ = None;
                let mut timestamp__ = None;
                let mut file_name__ = None;
                let mut file_size__ = None;
                let mut file_modification_timestamp__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Timestamp => {
                            if timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("timestamp"));
                            }
                            timestamp__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::FileName => {
                            if file_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fileName"));
                            }
                            file_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FileSize => {
                            if file_size__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fileSize"));
                            }
                            file_size__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::FileModificationTimestamp => {
                            if file_modification_timestamp__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fileModificationTimestamp"));
                            }
                            file_modification_timestamp__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CommitInfo {
                    version: version__.unwrap_or_default(),
                    timestamp: timestamp__.unwrap_or_default(),
                    file_name: file_name__.unwrap_or_default(),
                    file_size: file_size__.unwrap_or_default(),
                    file_modification_timestamp: file_modification_timestamp__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.delta_commits.v1.CommitInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CommitRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_id.is_empty() {
            len += 1;
        }
        if !self.table_uri.is_empty() {
            len += 1;
        }
        if self.commit_info.is_some() {
            len += 1;
        }
        if self.latest_backfilled_version.is_some() {
            len += 1;
        }
        if self.metadata.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.delta_commits.v1.CommitRequest", len)?;
        if !self.table_id.is_empty() {
            struct_ser.serialize_field("table_id", &self.table_id)?;
        }
        if !self.table_uri.is_empty() {
            struct_ser.serialize_field("table_uri", &self.table_uri)?;
        }
        if let Some(v) = self.commit_info.as_ref() {
            struct_ser.serialize_field("commit_info", v)?;
        }
        if let Some(v) = self.latest_backfilled_version.as_ref() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("latest_backfilled_version", ToString::to_string(&v).as_str())?;
        }
        if let Some(v) = self.metadata.as_ref() {
            struct_ser.serialize_field("metadata", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CommitRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_id",
            "tableId",
            "table_uri",
            "tableUri",
            "commit_info",
            "commitInfo",
            "latest_backfilled_version",
            "latestBackfilledVersion",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableId,
            TableUri,
            CommitInfo,
            LatestBackfilledVersion,
            Metadata,
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
                            "tableId" | "table_id" => Ok(GeneratedField::TableId),
                            "tableUri" | "table_uri" => Ok(GeneratedField::TableUri),
                            "commitInfo" | "commit_info" => Ok(GeneratedField::CommitInfo),
                            "latestBackfilledVersion" | "latest_backfilled_version" => Ok(GeneratedField::LatestBackfilledVersion),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CommitRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.delta_commits.v1.CommitRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CommitRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_id__ = None;
                let mut table_uri__ = None;
                let mut commit_info__ = None;
                let mut latest_backfilled_version__ = None;
                let mut metadata__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TableId => {
                            if table_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableId"));
                            }
                            table_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TableUri => {
                            if table_uri__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableUri"));
                            }
                            table_uri__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CommitInfo => {
                            if commit_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commitInfo"));
                            }
                            commit_info__ = map_.next_value()?;
                        }
                        GeneratedField::LatestBackfilledVersion => {
                            if latest_backfilled_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("latestBackfilledVersion"));
                            }
                            latest_backfilled_version__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CommitRequest {
                    table_id: table_id__.unwrap_or_default(),
                    table_uri: table_uri__.unwrap_or_default(),
                    commit_info: commit_info__,
                    latest_backfilled_version: latest_backfilled_version__,
                    metadata: metadata__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.delta_commits.v1.CommitRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetCommitsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.table_id.is_empty() {
            len += 1;
        }
        if !self.table_uri.is_empty() {
            len += 1;
        }
        if self.start_version != 0 {
            len += 1;
        }
        if self.end_version.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.delta_commits.v1.GetCommitsRequest", len)?;
        if !self.table_id.is_empty() {
            struct_ser.serialize_field("table_id", &self.table_id)?;
        }
        if !self.table_uri.is_empty() {
            struct_ser.serialize_field("table_uri", &self.table_uri)?;
        }
        if self.start_version != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("start_version", ToString::to_string(&self.start_version).as_str())?;
        }
        if let Some(v) = self.end_version.as_ref() {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("end_version", ToString::to_string(&v).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetCommitsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_id",
            "tableId",
            "table_uri",
            "tableUri",
            "start_version",
            "startVersion",
            "end_version",
            "endVersion",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableId,
            TableUri,
            StartVersion,
            EndVersion,
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
                            "tableId" | "table_id" => Ok(GeneratedField::TableId),
                            "tableUri" | "table_uri" => Ok(GeneratedField::TableUri),
                            "startVersion" | "start_version" => Ok(GeneratedField::StartVersion),
                            "endVersion" | "end_version" => Ok(GeneratedField::EndVersion),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetCommitsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.delta_commits.v1.GetCommitsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetCommitsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_id__ = None;
                let mut table_uri__ = None;
                let mut start_version__ = None;
                let mut end_version__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TableId => {
                            if table_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableId"));
                            }
                            table_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TableUri => {
                            if table_uri__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableUri"));
                            }
                            table_uri__ = Some(map_.next_value()?);
                        }
                        GeneratedField::StartVersion => {
                            if start_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("startVersion"));
                            }
                            start_version__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::EndVersion => {
                            if end_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("endVersion"));
                            }
                            end_version__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GetCommitsRequest {
                    table_id: table_id__.unwrap_or_default(),
                    table_uri: table_uri__.unwrap_or_default(),
                    start_version: start_version__.unwrap_or_default(),
                    end_version: end_version__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.delta_commits.v1.GetCommitsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetCommitsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.commits.is_empty() {
            len += 1;
        }
        if self.latest_table_version != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.delta_commits.v1.GetCommitsResponse", len)?;
        if !self.commits.is_empty() {
            struct_ser.serialize_field("commits", &self.commits)?;
        }
        if self.latest_table_version != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("latest_table_version", ToString::to_string(&self.latest_table_version).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetCommitsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "commits",
            "latest_table_version",
            "latestTableVersion",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Commits,
            LatestTableVersion,
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
                            "commits" => Ok(GeneratedField::Commits),
                            "latestTableVersion" | "latest_table_version" => Ok(GeneratedField::LatestTableVersion),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetCommitsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.delta_commits.v1.GetCommitsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetCommitsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut commits__ = None;
                let mut latest_table_version__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Commits => {
                            if commits__.is_some() {
                                return Err(serde::de::Error::duplicate_field("commits"));
                            }
                            commits__ = Some(map_.next_value()?);
                        }
                        GeneratedField::LatestTableVersion => {
                            if latest_table_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("latestTableVersion"));
                            }
                            latest_table_version__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GetCommitsResponse {
                    commits: commits__.unwrap_or_default(),
                    latest_table_version: latest_table_version__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.delta_commits.v1.GetCommitsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Metadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id.is_some() {
            len += 1;
        }
        if self.schema_string.is_some() {
            len += 1;
        }
        if !self.configuration.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.delta_commits.v1.Metadata", len)?;
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if let Some(v) = self.schema_string.as_ref() {
            struct_ser.serialize_field("schema_string", v)?;
        }
        if !self.configuration.is_empty() {
            struct_ser.serialize_field("configuration", &self.configuration)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Metadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "schema_string",
            "schemaString",
            "configuration",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            SchemaString,
            Configuration,
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
                            "id" => Ok(GeneratedField::Id),
                            "schemaString" | "schema_string" => Ok(GeneratedField::SchemaString),
                            "configuration" => Ok(GeneratedField::Configuration),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Metadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.delta_commits.v1.Metadata")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Metadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut schema_string__ = None;
                let mut configuration__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::SchemaString => {
                            if schema_string__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaString"));
                            }
                            schema_string__ = map_.next_value()?;
                        }
                        GeneratedField::Configuration => {
                            if configuration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("configuration"));
                            }
                            configuration__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Metadata {
                    id: id__,
                    schema_string: schema_string__,
                    configuration: configuration__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.delta_commits.v1.Metadata", FIELDS, GeneratedVisitor)
    }
}
