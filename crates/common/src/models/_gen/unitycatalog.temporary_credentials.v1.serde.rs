// @generated
impl serde::Serialize for AwsTemporaryCredentials {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.access_key_id.is_empty() {
            len += 1;
        }
        if !self.access_point.is_empty() {
            len += 1;
        }
        if !self.secret_access_key.is_empty() {
            len += 1;
        }
        if !self.session_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.temporary_credentials.v1.AwsTemporaryCredentials", len)?;
        if !self.access_key_id.is_empty() {
            struct_ser.serialize_field("access_key_id", &self.access_key_id)?;
        }
        if !self.access_point.is_empty() {
            struct_ser.serialize_field("access_point", &self.access_point)?;
        }
        if !self.secret_access_key.is_empty() {
            struct_ser.serialize_field("secret_access_key", &self.secret_access_key)?;
        }
        if !self.session_token.is_empty() {
            struct_ser.serialize_field("session_token", &self.session_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AwsTemporaryCredentials {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "access_key_id",
            "accessKeyId",
            "access_point",
            "accessPoint",
            "secret_access_key",
            "secretAccessKey",
            "session_token",
            "sessionToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccessKeyId,
            AccessPoint,
            SecretAccessKey,
            SessionToken,
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
                            "accessKeyId" | "access_key_id" => Ok(GeneratedField::AccessKeyId),
                            "accessPoint" | "access_point" => Ok(GeneratedField::AccessPoint),
                            "secretAccessKey" | "secret_access_key" => Ok(GeneratedField::SecretAccessKey),
                            "sessionToken" | "session_token" => Ok(GeneratedField::SessionToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AwsTemporaryCredentials;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.temporary_credentials.v1.AwsTemporaryCredentials")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AwsTemporaryCredentials, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut access_key_id__ = None;
                let mut access_point__ = None;
                let mut secret_access_key__ = None;
                let mut session_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AccessKeyId => {
                            if access_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accessKeyId"));
                            }
                            access_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AccessPoint => {
                            if access_point__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accessPoint"));
                            }
                            access_point__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SecretAccessKey => {
                            if secret_access_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("secretAccessKey"));
                            }
                            secret_access_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SessionToken => {
                            if session_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sessionToken"));
                            }
                            session_token__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AwsTemporaryCredentials {
                    access_key_id: access_key_id__.unwrap_or_default(),
                    access_point: access_point__.unwrap_or_default(),
                    secret_access_key: secret_access_key__.unwrap_or_default(),
                    session_token: session_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.temporary_credentials.v1.AwsTemporaryCredentials", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AzureAad {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.aad_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.temporary_credentials.v1.AzureAad", len)?;
        if !self.aad_token.is_empty() {
            struct_ser.serialize_field("aad_token", &self.aad_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AzureAad {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "aad_token",
            "aadToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AadToken,
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
                            "aadToken" | "aad_token" => Ok(GeneratedField::AadToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AzureAad;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.temporary_credentials.v1.AzureAad")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AzureAad, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut aad_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AadToken => {
                            if aad_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("aadToken"));
                            }
                            aad_token__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AzureAad {
                    aad_token: aad_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.temporary_credentials.v1.AzureAad", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AzureUserDelegationSas {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.sas_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.temporary_credentials.v1.AzureUserDelegationSas", len)?;
        if !self.sas_token.is_empty() {
            struct_ser.serialize_field("sas_token", &self.sas_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AzureUserDelegationSas {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sas_token",
            "sasToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SasToken,
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
                            "sasToken" | "sas_token" => Ok(GeneratedField::SasToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AzureUserDelegationSas;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.temporary_credentials.v1.AzureUserDelegationSas")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AzureUserDelegationSas, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sas_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SasToken => {
                            if sas_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sasToken"));
                            }
                            sas_token__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AzureUserDelegationSas {
                    sas_token: sas_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.temporary_credentials.v1.AzureUserDelegationSas", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GcpOauthToken {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.oauth_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.temporary_credentials.v1.GcpOauthToken", len)?;
        if !self.oauth_token.is_empty() {
            struct_ser.serialize_field("oauth_token", &self.oauth_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GcpOauthToken {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "oauth_token",
            "oauthToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OauthToken,
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
                            "oauthToken" | "oauth_token" => Ok(GeneratedField::OauthToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GcpOauthToken;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.temporary_credentials.v1.GcpOauthToken")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GcpOauthToken, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut oauth_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OauthToken => {
                            if oauth_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("oauthToken"));
                            }
                            oauth_token__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GcpOauthToken {
                    oauth_token: oauth_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.temporary_credentials.v1.GcpOauthToken", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateTemporaryPathCredentialsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.url.is_empty() {
            len += 1;
        }
        if self.operation != 0 {
            len += 1;
        }
        if self.dry_run.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.temporary_credentials.v1.GenerateTemporaryPathCredentialsRequest", len)?;
        if !self.url.is_empty() {
            struct_ser.serialize_field("url", &self.url)?;
        }
        if self.operation != 0 {
            let v = generate_temporary_path_credentials_request::Operation::try_from(self.operation)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.operation)))?;
            struct_ser.serialize_field("operation", &v)?;
        }
        if let Some(v) = self.dry_run.as_ref() {
            struct_ser.serialize_field("dry_run", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateTemporaryPathCredentialsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "url",
            "operation",
            "dry_run",
            "dryRun",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Url,
            Operation,
            DryRun,
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
                            "url" => Ok(GeneratedField::Url),
                            "operation" => Ok(GeneratedField::Operation),
                            "dryRun" | "dry_run" => Ok(GeneratedField::DryRun),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateTemporaryPathCredentialsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.temporary_credentials.v1.GenerateTemporaryPathCredentialsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateTemporaryPathCredentialsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut url__ = None;
                let mut operation__ = None;
                let mut dry_run__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Url => {
                            if url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Operation => {
                            if operation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("operation"));
                            }
                            operation__ = Some(map_.next_value::<generate_temporary_path_credentials_request::Operation>()? as i32);
                        }
                        GeneratedField::DryRun => {
                            if dry_run__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dryRun"));
                            }
                            dry_run__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenerateTemporaryPathCredentialsRequest {
                    url: url__.unwrap_or_default(),
                    operation: operation__.unwrap_or_default(),
                    dry_run: dry_run__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.temporary_credentials.v1.GenerateTemporaryPathCredentialsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for generate_temporary_path_credentials_request::Operation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "UNSPECIFIED",
            Self::PathRead => "PATH_READ",
            Self::PathReadWrite => "PATH_READ_WRITE",
            Self::PathCreateTable => "PATH_CREATE_TABLE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for generate_temporary_path_credentials_request::Operation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "UNSPECIFIED",
            "PATH_READ",
            "PATH_READ_WRITE",
            "PATH_CREATE_TABLE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = generate_temporary_path_credentials_request::Operation;

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
                    "UNSPECIFIED" => Ok(generate_temporary_path_credentials_request::Operation::Unspecified),
                    "PATH_READ" => Ok(generate_temporary_path_credentials_request::Operation::PathRead),
                    "PATH_READ_WRITE" => Ok(generate_temporary_path_credentials_request::Operation::PathReadWrite),
                    "PATH_CREATE_TABLE" => Ok(generate_temporary_path_credentials_request::Operation::PathCreateTable),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateTemporaryTableCredentialsRequest {
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
        if self.operation != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.temporary_credentials.v1.GenerateTemporaryTableCredentialsRequest", len)?;
        if !self.table_id.is_empty() {
            struct_ser.serialize_field("table_id", &self.table_id)?;
        }
        if self.operation != 0 {
            let v = generate_temporary_table_credentials_request::Operation::try_from(self.operation)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.operation)))?;
            struct_ser.serialize_field("operation", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateTemporaryTableCredentialsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "table_id",
            "tableId",
            "operation",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            TableId,
            Operation,
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
                            "operation" => Ok(GeneratedField::Operation),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateTemporaryTableCredentialsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.temporary_credentials.v1.GenerateTemporaryTableCredentialsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateTemporaryTableCredentialsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut table_id__ = None;
                let mut operation__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::TableId => {
                            if table_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tableId"));
                            }
                            table_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Operation => {
                            if operation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("operation"));
                            }
                            operation__ = Some(map_.next_value::<generate_temporary_table_credentials_request::Operation>()? as i32);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenerateTemporaryTableCredentialsRequest {
                    table_id: table_id__.unwrap_or_default(),
                    operation: operation__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.temporary_credentials.v1.GenerateTemporaryTableCredentialsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for generate_temporary_table_credentials_request::Operation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "UNSPECIFIED",
            Self::Read => "READ",
            Self::ReadWrite => "READ_WRITE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for generate_temporary_table_credentials_request::Operation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "UNSPECIFIED",
            "READ",
            "READ_WRITE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = generate_temporary_table_credentials_request::Operation;

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
                    "UNSPECIFIED" => Ok(generate_temporary_table_credentials_request::Operation::Unspecified),
                    "READ" => Ok(generate_temporary_table_credentials_request::Operation::Read),
                    "READ_WRITE" => Ok(generate_temporary_table_credentials_request::Operation::ReadWrite),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for GenerateTemporaryVolumeCredentialsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.volume_id.is_empty() {
            len += 1;
        }
        if self.operation != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.temporary_credentials.v1.GenerateTemporaryVolumeCredentialsRequest", len)?;
        if !self.volume_id.is_empty() {
            struct_ser.serialize_field("volume_id", &self.volume_id)?;
        }
        if self.operation != 0 {
            let v = generate_temporary_volume_credentials_request::Operation::try_from(self.operation)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.operation)))?;
            struct_ser.serialize_field("operation", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenerateTemporaryVolumeCredentialsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "volume_id",
            "volumeId",
            "operation",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VolumeId,
            Operation,
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
                            "volumeId" | "volume_id" => Ok(GeneratedField::VolumeId),
                            "operation" => Ok(GeneratedField::Operation),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenerateTemporaryVolumeCredentialsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.temporary_credentials.v1.GenerateTemporaryVolumeCredentialsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GenerateTemporaryVolumeCredentialsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut volume_id__ = None;
                let mut operation__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::VolumeId => {
                            if volume_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("volumeId"));
                            }
                            volume_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Operation => {
                            if operation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("operation"));
                            }
                            operation__ = Some(map_.next_value::<generate_temporary_volume_credentials_request::Operation>()? as i32);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(GenerateTemporaryVolumeCredentialsRequest {
                    volume_id: volume_id__.unwrap_or_default(),
                    operation: operation__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.temporary_credentials.v1.GenerateTemporaryVolumeCredentialsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for generate_temporary_volume_credentials_request::Operation {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "UNSPECIFIED",
            Self::ReadVolume => "READ_VOLUME",
            Self::WriteVolume => "WRITE_VOLUME",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for generate_temporary_volume_credentials_request::Operation {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "UNSPECIFIED",
            "READ_VOLUME",
            "WRITE_VOLUME",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = generate_temporary_volume_credentials_request::Operation;

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
                    "UNSPECIFIED" => Ok(generate_temporary_volume_credentials_request::Operation::Unspecified),
                    "READ_VOLUME" => Ok(generate_temporary_volume_credentials_request::Operation::ReadVolume),
                    "WRITE_VOLUME" => Ok(generate_temporary_volume_credentials_request::Operation::WriteVolume),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for R2TemporaryCredentials {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.access_key_id.is_empty() {
            len += 1;
        }
        if !self.secret_access_key.is_empty() {
            len += 1;
        }
        if !self.session_token.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.temporary_credentials.v1.R2TemporaryCredentials", len)?;
        if !self.access_key_id.is_empty() {
            struct_ser.serialize_field("access_key_id", &self.access_key_id)?;
        }
        if !self.secret_access_key.is_empty() {
            struct_ser.serialize_field("secret_access_key", &self.secret_access_key)?;
        }
        if !self.session_token.is_empty() {
            struct_ser.serialize_field("session_token", &self.session_token)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for R2TemporaryCredentials {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "access_key_id",
            "accessKeyId",
            "secret_access_key",
            "secretAccessKey",
            "session_token",
            "sessionToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccessKeyId,
            SecretAccessKey,
            SessionToken,
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
                            "accessKeyId" | "access_key_id" => Ok(GeneratedField::AccessKeyId),
                            "secretAccessKey" | "secret_access_key" => Ok(GeneratedField::SecretAccessKey),
                            "sessionToken" | "session_token" => Ok(GeneratedField::SessionToken),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = R2TemporaryCredentials;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.temporary_credentials.v1.R2TemporaryCredentials")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<R2TemporaryCredentials, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut access_key_id__ = None;
                let mut secret_access_key__ = None;
                let mut session_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AccessKeyId => {
                            if access_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accessKeyId"));
                            }
                            access_key_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SecretAccessKey => {
                            if secret_access_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("secretAccessKey"));
                            }
                            secret_access_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SessionToken => {
                            if session_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sessionToken"));
                            }
                            session_token__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(R2TemporaryCredentials {
                    access_key_id: access_key_id__.unwrap_or_default(),
                    secret_access_key: secret_access_key__.unwrap_or_default(),
                    session_token: session_token__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.temporary_credentials.v1.R2TemporaryCredentials", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TemporaryCredential {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.expiration_time != 0 {
            len += 1;
        }
        if !self.url.is_empty() {
            len += 1;
        }
        if self.credentials.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.temporary_credentials.v1.TemporaryCredential", len)?;
        if self.expiration_time != 0 {
            #[allow(clippy::needless_borrow)]
            #[allow(clippy::needless_borrows_for_generic_args)]
            struct_ser.serialize_field("expiration_time", ToString::to_string(&self.expiration_time).as_str())?;
        }
        if !self.url.is_empty() {
            struct_ser.serialize_field("url", &self.url)?;
        }
        if let Some(v) = self.credentials.as_ref() {
            match v {
                temporary_credential::Credentials::AzureUserDelegationSas(v) => {
                    struct_ser.serialize_field("azure_user_delegation_sas", v)?;
                }
                temporary_credential::Credentials::AzureAad(v) => {
                    struct_ser.serialize_field("azure_aad", v)?;
                }
                temporary_credential::Credentials::AwsTempCredentials(v) => {
                    struct_ser.serialize_field("aws_temp_credentials", v)?;
                }
                temporary_credential::Credentials::GcpOauthToken(v) => {
                    struct_ser.serialize_field("gcp_oauth_token", v)?;
                }
                temporary_credential::Credentials::R2TempCredentials(v) => {
                    struct_ser.serialize_field("r2_temp_credentials", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TemporaryCredential {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "expiration_time",
            "expirationTime",
            "url",
            "azure_user_delegation_sas",
            "azureUserDelegationSas",
            "azure_aad",
            "azureAad",
            "aws_temp_credentials",
            "awsTempCredentials",
            "gcp_oauth_token",
            "gcpOauthToken",
            "r2_temp_credentials",
            "r2TempCredentials",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExpirationTime,
            Url,
            AzureUserDelegationSas,
            AzureAad,
            AwsTempCredentials,
            GcpOauthToken,
            R2TempCredentials,
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
                            "expirationTime" | "expiration_time" => Ok(GeneratedField::ExpirationTime),
                            "url" => Ok(GeneratedField::Url),
                            "azureUserDelegationSas" | "azure_user_delegation_sas" => Ok(GeneratedField::AzureUserDelegationSas),
                            "azureAad" | "azure_aad" => Ok(GeneratedField::AzureAad),
                            "awsTempCredentials" | "aws_temp_credentials" => Ok(GeneratedField::AwsTempCredentials),
                            "gcpOauthToken" | "gcp_oauth_token" => Ok(GeneratedField::GcpOauthToken),
                            "r2TempCredentials" | "r2_temp_credentials" => Ok(GeneratedField::R2TempCredentials),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TemporaryCredential;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.temporary_credentials.v1.TemporaryCredential")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<TemporaryCredential, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut expiration_time__ = None;
                let mut url__ = None;
                let mut credentials__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ExpirationTime => {
                            if expiration_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("expirationTime"));
                            }
                            expiration_time__ = 
                                Some(map_.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Url => {
                            if url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AzureUserDelegationSas => {
                            if credentials__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureUserDelegationSas"));
                            }
                            credentials__ = map_.next_value::<::std::option::Option<_>>()?.map(temporary_credential::Credentials::AzureUserDelegationSas)
;
                        }
                        GeneratedField::AzureAad => {
                            if credentials__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureAad"));
                            }
                            credentials__ = map_.next_value::<::std::option::Option<_>>()?.map(temporary_credential::Credentials::AzureAad)
;
                        }
                        GeneratedField::AwsTempCredentials => {
                            if credentials__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awsTempCredentials"));
                            }
                            credentials__ = map_.next_value::<::std::option::Option<_>>()?.map(temporary_credential::Credentials::AwsTempCredentials)
;
                        }
                        GeneratedField::GcpOauthToken => {
                            if credentials__.is_some() {
                                return Err(serde::de::Error::duplicate_field("gcpOauthToken"));
                            }
                            credentials__ = map_.next_value::<::std::option::Option<_>>()?.map(temporary_credential::Credentials::GcpOauthToken)
;
                        }
                        GeneratedField::R2TempCredentials => {
                            if credentials__.is_some() {
                                return Err(serde::de::Error::duplicate_field("r2TempCredentials"));
                            }
                            credentials__ = map_.next_value::<::std::option::Option<_>>()?.map(temporary_credential::Credentials::R2TempCredentials)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(TemporaryCredential {
                    expiration_time: expiration_time__.unwrap_or_default(),
                    url: url__.unwrap_or_default(),
                    credentials: credentials__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.temporary_credentials.v1.TemporaryCredential", FIELDS, GeneratedVisitor)
    }
}
