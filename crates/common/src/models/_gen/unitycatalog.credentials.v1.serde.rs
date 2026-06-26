// @generated
impl serde::Serialize for AwsIamRole {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.external_id.is_empty() {
            len += 1;
        }
        if !self.role_arn.is_empty() {
            len += 1;
        }
        if !self.unity_catalog_iam_arn.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.AwsIamRole", len)?;
        if !self.external_id.is_empty() {
            struct_ser.serialize_field("external_id", &self.external_id)?;
        }
        if !self.role_arn.is_empty() {
            struct_ser.serialize_field("role_arn", &self.role_arn)?;
        }
        if !self.unity_catalog_iam_arn.is_empty() {
            struct_ser.serialize_field("unity_catalog_iam_arn", &self.unity_catalog_iam_arn)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AwsIamRole {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "external_id",
            "externalId",
            "role_arn",
            "roleArn",
            "unity_catalog_iam_arn",
            "unityCatalogIamArn",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ExternalId,
            RoleArn,
            UnityCatalogIamArn,
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
                            "externalId" | "external_id" => Ok(GeneratedField::ExternalId),
                            "roleArn" | "role_arn" => Ok(GeneratedField::RoleArn),
                            "unityCatalogIamArn" | "unity_catalog_iam_arn" => Ok(GeneratedField::UnityCatalogIamArn),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AwsIamRole;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.AwsIamRole")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AwsIamRole, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut external_id__ = None;
                let mut role_arn__ = None;
                let mut unity_catalog_iam_arn__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::ExternalId => {
                            if external_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("externalId"));
                            }
                            external_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::RoleArn => {
                            if role_arn__.is_some() {
                                return Err(serde::de::Error::duplicate_field("roleArn"));
                            }
                            role_arn__ = Some(map_.next_value()?);
                        }
                        GeneratedField::UnityCatalogIamArn => {
                            if unity_catalog_iam_arn__.is_some() {
                                return Err(serde::de::Error::duplicate_field("unityCatalogIamArn"));
                            }
                            unity_catalog_iam_arn__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AwsIamRole {
                    external_id: external_id__.unwrap_or_default(),
                    role_arn: role_arn__.unwrap_or_default(),
                    unity_catalog_iam_arn: unity_catalog_iam_arn__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.AwsIamRole", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AwsIamRoleConfig {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.role_arn.is_empty() {
            len += 1;
        }
        if self.region.is_some() {
            len += 1;
        }
        if self.access_key_id.is_some() {
            len += 1;
        }
        if self.secret_access_key.is_some() {
            len += 1;
        }
        if self.session_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.AwsIamRoleConfig", len)?;
        if !self.role_arn.is_empty() {
            struct_ser.serialize_field("role_arn", &self.role_arn)?;
        }
        if let Some(v) = self.region.as_ref() {
            struct_ser.serialize_field("region", v)?;
        }
        if let Some(v) = self.access_key_id.as_ref() {
            struct_ser.serialize_field("access_key_id", v)?;
        }
        if let Some(v) = self.secret_access_key.as_ref() {
            struct_ser.serialize_field("secret_access_key", v)?;
        }
        if let Some(v) = self.session_token.as_ref() {
            struct_ser.serialize_field("session_token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AwsIamRoleConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "role_arn",
            "roleArn",
            "region",
            "access_key_id",
            "accessKeyId",
            "secret_access_key",
            "secretAccessKey",
            "session_token",
            "sessionToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RoleArn,
            Region,
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
                            "roleArn" | "role_arn" => Ok(GeneratedField::RoleArn),
                            "region" => Ok(GeneratedField::Region),
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
            type Value = AwsIamRoleConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.AwsIamRoleConfig")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AwsIamRoleConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut role_arn__ = None;
                let mut region__ = None;
                let mut access_key_id__ = None;
                let mut secret_access_key__ = None;
                let mut session_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::RoleArn => {
                            if role_arn__.is_some() {
                                return Err(serde::de::Error::duplicate_field("roleArn"));
                            }
                            role_arn__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Region => {
                            if region__.is_some() {
                                return Err(serde::de::Error::duplicate_field("region"));
                            }
                            region__ = map_.next_value()?;
                        }
                        GeneratedField::AccessKeyId => {
                            if access_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accessKeyId"));
                            }
                            access_key_id__ = map_.next_value()?;
                        }
                        GeneratedField::SecretAccessKey => {
                            if secret_access_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("secretAccessKey"));
                            }
                            secret_access_key__ = map_.next_value()?;
                        }
                        GeneratedField::SessionToken => {
                            if session_token__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sessionToken"));
                            }
                            session_token__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AwsIamRoleConfig {
                    role_arn: role_arn__.unwrap_or_default(),
                    region: region__,
                    access_key_id: access_key_id__,
                    secret_access_key: secret_access_key__,
                    session_token: session_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.AwsIamRoleConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AzureManagedIdentity {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.access_connector_id.is_empty() {
            len += 1;
        }
        if self.credential_id.is_some() {
            len += 1;
        }
        if self.managed_identity_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.AzureManagedIdentity", len)?;
        if !self.access_connector_id.is_empty() {
            struct_ser.serialize_field("access_connector_id", &self.access_connector_id)?;
        }
        if let Some(v) = self.credential_id.as_ref() {
            struct_ser.serialize_field("credential_id", v)?;
        }
        if let Some(v) = self.managed_identity_id.as_ref() {
            struct_ser.serialize_field("managed_identity_id", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AzureManagedIdentity {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "access_connector_id",
            "accessConnectorId",
            "credential_id",
            "credentialId",
            "managed_identity_id",
            "managedIdentityId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccessConnectorId,
            CredentialId,
            ManagedIdentityId,
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
                            "accessConnectorId" | "access_connector_id" => Ok(GeneratedField::AccessConnectorId),
                            "credentialId" | "credential_id" => Ok(GeneratedField::CredentialId),
                            "managedIdentityId" | "managed_identity_id" => Ok(GeneratedField::ManagedIdentityId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AzureManagedIdentity;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.AzureManagedIdentity")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AzureManagedIdentity, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut access_connector_id__ = None;
                let mut credential_id__ = None;
                let mut managed_identity_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AccessConnectorId => {
                            if access_connector_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accessConnectorId"));
                            }
                            access_connector_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::CredentialId => {
                            if credential_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credentialId"));
                            }
                            credential_id__ = map_.next_value()?;
                        }
                        GeneratedField::ManagedIdentityId => {
                            if managed_identity_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("managedIdentityId"));
                            }
                            managed_identity_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AzureManagedIdentity {
                    access_connector_id: access_connector_id__.unwrap_or_default(),
                    credential_id: credential_id__,
                    managed_identity_id: managed_identity_id__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.AzureManagedIdentity", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AzureServicePrincipal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.directory_id.is_empty() {
            len += 1;
        }
        if !self.application_id.is_empty() {
            len += 1;
        }
        if self.credential.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.AzureServicePrincipal", len)?;
        if !self.directory_id.is_empty() {
            struct_ser.serialize_field("directory_id", &self.directory_id)?;
        }
        if !self.application_id.is_empty() {
            struct_ser.serialize_field("application_id", &self.application_id)?;
        }
        if let Some(v) = self.credential.as_ref() {
            match v {
                azure_service_principal::Credential::ClientSecret(v) => {
                    struct_ser.serialize_field("client_secret", v)?;
                }
                azure_service_principal::Credential::FederatedTokenFile(v) => {
                    struct_ser.serialize_field("federated_token_file", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AzureServicePrincipal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "directory_id",
            "directoryId",
            "application_id",
            "applicationId",
            "client_secret",
            "clientSecret",
            "federated_token_file",
            "federatedTokenFile",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            DirectoryId,
            ApplicationId,
            ClientSecret,
            FederatedTokenFile,
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
                            "directoryId" | "directory_id" => Ok(GeneratedField::DirectoryId),
                            "applicationId" | "application_id" => Ok(GeneratedField::ApplicationId),
                            "clientSecret" | "client_secret" => Ok(GeneratedField::ClientSecret),
                            "federatedTokenFile" | "federated_token_file" => Ok(GeneratedField::FederatedTokenFile),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AzureServicePrincipal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.AzureServicePrincipal")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AzureServicePrincipal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut directory_id__ = None;
                let mut application_id__ = None;
                let mut credential__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::DirectoryId => {
                            if directory_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("directoryId"));
                            }
                            directory_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ApplicationId => {
                            if application_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("applicationId"));
                            }
                            application_id__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ClientSecret => {
                            if credential__.is_some() {
                                return Err(serde::de::Error::duplicate_field("clientSecret"));
                            }
                            credential__ = map_.next_value::<::std::option::Option<_>>()?.map(azure_service_principal::Credential::ClientSecret);
                        }
                        GeneratedField::FederatedTokenFile => {
                            if credential__.is_some() {
                                return Err(serde::de::Error::duplicate_field("federatedTokenFile"));
                            }
                            credential__ = map_.next_value::<::std::option::Option<_>>()?.map(azure_service_principal::Credential::FederatedTokenFile);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AzureServicePrincipal {
                    directory_id: directory_id__.unwrap_or_default(),
                    application_id: application_id__.unwrap_or_default(),
                    credential: credential__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.AzureServicePrincipal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AzureStorageKey {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.account_name.is_empty() {
            len += 1;
        }
        if !self.account_key.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.AzureStorageKey", len)?;
        if !self.account_name.is_empty() {
            struct_ser.serialize_field("account_name", &self.account_name)?;
        }
        if !self.account_key.is_empty() {
            struct_ser.serialize_field("account_key", &self.account_key)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AzureStorageKey {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "account_name",
            "accountName",
            "account_key",
            "accountKey",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            AccountName,
            AccountKey,
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
                            "accountName" | "account_name" => Ok(GeneratedField::AccountName),
                            "accountKey" | "account_key" => Ok(GeneratedField::AccountKey),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AzureStorageKey;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.AzureStorageKey")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AzureStorageKey, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account_name__ = None;
                let mut account_key__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::AccountName => {
                            if account_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountName"));
                            }
                            account_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::AccountKey => {
                            if account_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("accountKey"));
                            }
                            account_key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(AzureStorageKey {
                    account_name: account_name__.unwrap_or_default(),
                    account_key: account_key__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.AzureStorageKey", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CreateCredentialRequest {
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
        if self.purpose != 0 {
            len += 1;
        }
        if self.comment.is_some() {
            len += 1;
        }
        if self.read_only.is_some() {
            len += 1;
        }
        if self.skip_validation.is_some() {
            len += 1;
        }
        if self.azure_service_principal.is_some() {
            len += 1;
        }
        if self.azure_managed_identity.is_some() {
            len += 1;
        }
        if self.azure_storage_key.is_some() {
            len += 1;
        }
        if self.aws_iam_role.is_some() {
            len += 1;
        }
        if self.databricks_gcp_service_account.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.CreateCredentialRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.purpose != 0 {
            let v = Purpose::try_from(self.purpose)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.purpose)))?;
            struct_ser.serialize_field("purpose", &v)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
        }
        if let Some(v) = self.read_only.as_ref() {
            struct_ser.serialize_field("read_only", v)?;
        }
        if let Some(v) = self.skip_validation.as_ref() {
            struct_ser.serialize_field("skip_validation", v)?;
        }
        if let Some(v) = self.azure_service_principal.as_ref() {
            struct_ser.serialize_field("azure_service_principal", v)?;
        }
        if let Some(v) = self.azure_managed_identity.as_ref() {
            struct_ser.serialize_field("azure_managed_identity", v)?;
        }
        if let Some(v) = self.azure_storage_key.as_ref() {
            struct_ser.serialize_field("azure_storage_key", v)?;
        }
        if let Some(v) = self.aws_iam_role.as_ref() {
            struct_ser.serialize_field("aws_iam_role", v)?;
        }
        if let Some(v) = self.databricks_gcp_service_account.as_ref() {
            struct_ser.serialize_field("databricks_gcp_service_account", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateCredentialRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "purpose",
            "comment",
            "read_only",
            "readOnly",
            "skip_validation",
            "skipValidation",
            "azure_service_principal",
            "azureServicePrincipal",
            "azure_managed_identity",
            "azureManagedIdentity",
            "azure_storage_key",
            "azureStorageKey",
            "aws_iam_role",
            "awsIamRole",
            "databricks_gcp_service_account",
            "databricksGcpServiceAccount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Purpose,
            Comment,
            ReadOnly,
            SkipValidation,
            AzureServicePrincipal,
            AzureManagedIdentity,
            AzureStorageKey,
            AwsIamRole,
            DatabricksGcpServiceAccount,
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
                            "purpose" => Ok(GeneratedField::Purpose),
                            "comment" => Ok(GeneratedField::Comment),
                            "readOnly" | "read_only" => Ok(GeneratedField::ReadOnly),
                            "skipValidation" | "skip_validation" => Ok(GeneratedField::SkipValidation),
                            "azureServicePrincipal" | "azure_service_principal" => Ok(GeneratedField::AzureServicePrincipal),
                            "azureManagedIdentity" | "azure_managed_identity" => Ok(GeneratedField::AzureManagedIdentity),
                            "azureStorageKey" | "azure_storage_key" => Ok(GeneratedField::AzureStorageKey),
                            "awsIamRole" | "aws_iam_role" => Ok(GeneratedField::AwsIamRole),
                            "databricksGcpServiceAccount" | "databricks_gcp_service_account" => Ok(GeneratedField::DatabricksGcpServiceAccount),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateCredentialRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.CreateCredentialRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateCredentialRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut purpose__ = None;
                let mut comment__ = None;
                let mut read_only__ = None;
                let mut skip_validation__ = None;
                let mut azure_service_principal__ = None;
                let mut azure_managed_identity__ = None;
                let mut azure_storage_key__ = None;
                let mut aws_iam_role__ = None;
                let mut databricks_gcp_service_account__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Purpose => {
                            if purpose__.is_some() {
                                return Err(serde::de::Error::duplicate_field("purpose"));
                            }
                            purpose__ = Some(map_.next_value::<Purpose>()? as i32);
                        }
                        GeneratedField::Comment => {
                            if comment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("comment"));
                            }
                            comment__ = map_.next_value()?;
                        }
                        GeneratedField::ReadOnly => {
                            if read_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("readOnly"));
                            }
                            read_only__ = map_.next_value()?;
                        }
                        GeneratedField::SkipValidation => {
                            if skip_validation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("skipValidation"));
                            }
                            skip_validation__ = map_.next_value()?;
                        }
                        GeneratedField::AzureServicePrincipal => {
                            if azure_service_principal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureServicePrincipal"));
                            }
                            azure_service_principal__ = map_.next_value()?;
                        }
                        GeneratedField::AzureManagedIdentity => {
                            if azure_managed_identity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureManagedIdentity"));
                            }
                            azure_managed_identity__ = map_.next_value()?;
                        }
                        GeneratedField::AzureStorageKey => {
                            if azure_storage_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureStorageKey"));
                            }
                            azure_storage_key__ = map_.next_value()?;
                        }
                        GeneratedField::AwsIamRole => {
                            if aws_iam_role__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awsIamRole"));
                            }
                            aws_iam_role__ = map_.next_value()?;
                        }
                        GeneratedField::DatabricksGcpServiceAccount => {
                            if databricks_gcp_service_account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("databricksGcpServiceAccount"));
                            }
                            databricks_gcp_service_account__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CreateCredentialRequest {
                    name: name__.unwrap_or_default(),
                    purpose: purpose__.unwrap_or_default(),
                    comment: comment__,
                    read_only: read_only__,
                    skip_validation: skip_validation__,
                    azure_service_principal: azure_service_principal__,
                    azure_managed_identity: azure_managed_identity__,
                    azure_storage_key: azure_storage_key__,
                    aws_iam_role: aws_iam_role__,
                    databricks_gcp_service_account: databricks_gcp_service_account__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.CreateCredentialRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Credential {
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
        if self.id.is_some() {
            len += 1;
        }
        if self.purpose != 0 {
            len += 1;
        }
        if self.read_only {
            len += 1;
        }
        if self.comment.is_some() {
            len += 1;
        }
        if self.owner.is_some() {
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
        if self.used_for_managed_storage {
            len += 1;
        }
        if self.full_name.is_some() {
            len += 1;
        }
        if self.azure_service_principal.is_some() {
            len += 1;
        }
        if self.azure_managed_identity.is_some() {
            len += 1;
        }
        if self.azure_storage_key.is_some() {
            len += 1;
        }
        if self.aws_iam_role.is_some() {
            len += 1;
        }
        if self.databricks_gcp_service_account.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.Credential", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if self.purpose != 0 {
            let v = Purpose::try_from(self.purpose)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.purpose)))?;
            struct_ser.serialize_field("purpose", &v)?;
        }
        if self.read_only {
            struct_ser.serialize_field("read_only", &self.read_only)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
        }
        if let Some(v) = self.owner.as_ref() {
            struct_ser.serialize_field("owner", v)?;
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
        if self.used_for_managed_storage {
            struct_ser.serialize_field("used_for_managed_storage", &self.used_for_managed_storage)?;
        }
        if let Some(v) = self.full_name.as_ref() {
            struct_ser.serialize_field("full_name", v)?;
        }
        if let Some(v) = self.azure_service_principal.as_ref() {
            struct_ser.serialize_field("azure_service_principal", v)?;
        }
        if let Some(v) = self.azure_managed_identity.as_ref() {
            struct_ser.serialize_field("azure_managed_identity", v)?;
        }
        if let Some(v) = self.azure_storage_key.as_ref() {
            struct_ser.serialize_field("azure_storage_key", v)?;
        }
        if let Some(v) = self.aws_iam_role.as_ref() {
            struct_ser.serialize_field("aws_iam_role", v)?;
        }
        if let Some(v) = self.databricks_gcp_service_account.as_ref() {
            struct_ser.serialize_field("databricks_gcp_service_account", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Credential {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "id",
            "purpose",
            "read_only",
            "readOnly",
            "comment",
            "owner",
            "created_at",
            "createdAt",
            "created_by",
            "createdBy",
            "updated_at",
            "updatedAt",
            "updated_by",
            "updatedBy",
            "used_for_managed_storage",
            "usedForManagedStorage",
            "full_name",
            "fullName",
            "azure_service_principal",
            "azureServicePrincipal",
            "azure_managed_identity",
            "azureManagedIdentity",
            "azure_storage_key",
            "azureStorageKey",
            "aws_iam_role",
            "awsIamRole",
            "databricks_gcp_service_account",
            "databricksGcpServiceAccount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Id,
            Purpose,
            ReadOnly,
            Comment,
            Owner,
            CreatedAt,
            CreatedBy,
            UpdatedAt,
            UpdatedBy,
            UsedForManagedStorage,
            FullName,
            AzureServicePrincipal,
            AzureManagedIdentity,
            AzureStorageKey,
            AwsIamRole,
            DatabricksGcpServiceAccount,
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
                            "id" => Ok(GeneratedField::Id),
                            "purpose" => Ok(GeneratedField::Purpose),
                            "readOnly" | "read_only" => Ok(GeneratedField::ReadOnly),
                            "comment" => Ok(GeneratedField::Comment),
                            "owner" => Ok(GeneratedField::Owner),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "createdBy" | "created_by" => Ok(GeneratedField::CreatedBy),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            "updatedBy" | "updated_by" => Ok(GeneratedField::UpdatedBy),
                            "usedForManagedStorage" | "used_for_managed_storage" => Ok(GeneratedField::UsedForManagedStorage),
                            "fullName" | "full_name" => Ok(GeneratedField::FullName),
                            "azureServicePrincipal" | "azure_service_principal" => Ok(GeneratedField::AzureServicePrincipal),
                            "azureManagedIdentity" | "azure_managed_identity" => Ok(GeneratedField::AzureManagedIdentity),
                            "azureStorageKey" | "azure_storage_key" => Ok(GeneratedField::AzureStorageKey),
                            "awsIamRole" | "aws_iam_role" => Ok(GeneratedField::AwsIamRole),
                            "databricksGcpServiceAccount" | "databricks_gcp_service_account" => Ok(GeneratedField::DatabricksGcpServiceAccount),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Credential;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.Credential")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Credential, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut id__ = None;
                let mut purpose__ = None;
                let mut read_only__ = None;
                let mut comment__ = None;
                let mut owner__ = None;
                let mut created_at__ = None;
                let mut created_by__ = None;
                let mut updated_at__ = None;
                let mut updated_by__ = None;
                let mut used_for_managed_storage__ = None;
                let mut full_name__ = None;
                let mut azure_service_principal__ = None;
                let mut azure_managed_identity__ = None;
                let mut azure_storage_key__ = None;
                let mut aws_iam_role__ = None;
                let mut databricks_gcp_service_account__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = map_.next_value()?;
                        }
                        GeneratedField::Purpose => {
                            if purpose__.is_some() {
                                return Err(serde::de::Error::duplicate_field("purpose"));
                            }
                            purpose__ = Some(map_.next_value::<Purpose>()? as i32);
                        }
                        GeneratedField::ReadOnly => {
                            if read_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("readOnly"));
                            }
                            read_only__ = Some(map_.next_value()?);
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
                        GeneratedField::UsedForManagedStorage => {
                            if used_for_managed_storage__.is_some() {
                                return Err(serde::de::Error::duplicate_field("usedForManagedStorage"));
                            }
                            used_for_managed_storage__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullName => {
                            if full_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullName"));
                            }
                            full_name__ = map_.next_value()?;
                        }
                        GeneratedField::AzureServicePrincipal => {
                            if azure_service_principal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureServicePrincipal"));
                            }
                            azure_service_principal__ = map_.next_value()?;
                        }
                        GeneratedField::AzureManagedIdentity => {
                            if azure_managed_identity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureManagedIdentity"));
                            }
                            azure_managed_identity__ = map_.next_value()?;
                        }
                        GeneratedField::AzureStorageKey => {
                            if azure_storage_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureStorageKey"));
                            }
                            azure_storage_key__ = map_.next_value()?;
                        }
                        GeneratedField::AwsIamRole => {
                            if aws_iam_role__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awsIamRole"));
                            }
                            aws_iam_role__ = map_.next_value()?;
                        }
                        GeneratedField::DatabricksGcpServiceAccount => {
                            if databricks_gcp_service_account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("databricksGcpServiceAccount"));
                            }
                            databricks_gcp_service_account__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Credential {
                    name: name__.unwrap_or_default(),
                    id: id__,
                    purpose: purpose__.unwrap_or_default(),
                    read_only: read_only__.unwrap_or_default(),
                    comment: comment__,
                    owner: owner__,
                    created_at: created_at__,
                    created_by: created_by__,
                    updated_at: updated_at__,
                    updated_by: updated_by__,
                    used_for_managed_storage: used_for_managed_storage__.unwrap_or_default(),
                    full_name: full_name__,
                    azure_service_principal: azure_service_principal__,
                    azure_managed_identity: azure_managed_identity__,
                    azure_storage_key: azure_storage_key__,
                    aws_iam_role: aws_iam_role__,
                    databricks_gcp_service_account: databricks_gcp_service_account__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.Credential", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DatabricksGcpServiceAccount {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.credential_id.is_some() {
            len += 1;
        }
        if self.email.is_some() {
            len += 1;
        }
        if self.private_key_id.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.DatabricksGcpServiceAccount", len)?;
        if let Some(v) = self.credential_id.as_ref() {
            struct_ser.serialize_field("credential_id", v)?;
        }
        if let Some(v) = self.email.as_ref() {
            struct_ser.serialize_field("email", v)?;
        }
        if let Some(v) = self.private_key_id.as_ref() {
            struct_ser.serialize_field("private_key_id", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DatabricksGcpServiceAccount {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "credential_id",
            "credentialId",
            "email",
            "private_key_id",
            "privateKeyId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            CredentialId,
            Email,
            PrivateKeyId,
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
                            "credentialId" | "credential_id" => Ok(GeneratedField::CredentialId),
                            "email" => Ok(GeneratedField::Email),
                            "privateKeyId" | "private_key_id" => Ok(GeneratedField::PrivateKeyId),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DatabricksGcpServiceAccount;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.DatabricksGcpServiceAccount")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DatabricksGcpServiceAccount, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut credential_id__ = None;
                let mut email__ = None;
                let mut private_key_id__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::CredentialId => {
                            if credential_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credentialId"));
                            }
                            credential_id__ = map_.next_value()?;
                        }
                        GeneratedField::Email => {
                            if email__.is_some() {
                                return Err(serde::de::Error::duplicate_field("email"));
                            }
                            email__ = map_.next_value()?;
                        }
                        GeneratedField::PrivateKeyId => {
                            if private_key_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("privateKeyId"));
                            }
                            private_key_id__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DatabricksGcpServiceAccount {
                    credential_id: credential_id__,
                    email: email__,
                    private_key_id: private_key_id__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.DatabricksGcpServiceAccount", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteCredentialRequest {
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
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.DeleteCredentialRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteCredentialRequest {
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
            type Value = DeleteCredentialRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.DeleteCredentialRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteCredentialRequest, V::Error>
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
                Ok(DeleteCredentialRequest {
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.DeleteCredentialRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetCredentialRequest {
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
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.GetCredentialRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetCredentialRequest {
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
            type Value = GetCredentialRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.GetCredentialRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetCredentialRequest, V::Error>
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
                Ok(GetCredentialRequest {
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.GetCredentialRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListCredentialsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.purpose.is_some() {
            len += 1;
        }
        if self.max_results.is_some() {
            len += 1;
        }
        if self.page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.ListCredentialsRequest", len)?;
        if let Some(v) = self.purpose.as_ref() {
            let v = Purpose::try_from(*v)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", *v)))?;
            struct_ser.serialize_field("purpose", &v)?;
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
impl<'de> serde::Deserialize<'de> for ListCredentialsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "purpose",
            "max_results",
            "maxResults",
            "page_token",
            "pageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Purpose,
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
                            "purpose" => Ok(GeneratedField::Purpose),
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
            type Value = ListCredentialsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.ListCredentialsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListCredentialsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut purpose__ = None;
                let mut max_results__ = None;
                let mut page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Purpose => {
                            if purpose__.is_some() {
                                return Err(serde::de::Error::duplicate_field("purpose"));
                            }
                            purpose__ = map_.next_value::<::std::option::Option<Purpose>>()?.map(|x| x as i32);
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
                Ok(ListCredentialsRequest {
                    purpose: purpose__,
                    max_results: max_results__,
                    page_token: page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.ListCredentialsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListCredentialsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.credentials.is_empty() {
            len += 1;
        }
        if self.next_page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.ListCredentialsResponse", len)?;
        if !self.credentials.is_empty() {
            struct_ser.serialize_field("credentials", &self.credentials)?;
        }
        if let Some(v) = self.next_page_token.as_ref() {
            struct_ser.serialize_field("next_page_token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListCredentialsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "credentials",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Credentials,
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
                            "credentials" => Ok(GeneratedField::Credentials),
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
            type Value = ListCredentialsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.ListCredentialsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListCredentialsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut credentials__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Credentials => {
                            if credentials__.is_some() {
                                return Err(serde::de::Error::duplicate_field("credentials"));
                            }
                            credentials__ = Some(map_.next_value()?);
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
                Ok(ListCredentialsResponse {
                    credentials: credentials__.unwrap_or_default(),
                    next_page_token: next_page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.ListCredentialsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Purpose {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "PURPOSE_UNSPECIFIED",
            Self::Storage => "STORAGE",
            Self::Service => "SERVICE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Purpose {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "PURPOSE_UNSPECIFIED",
            "STORAGE",
            "SERVICE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Purpose;

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
                    "PURPOSE_UNSPECIFIED" => Ok(Purpose::Unspecified),
                    "STORAGE" => Ok(Purpose::Storage),
                    "SERVICE" => Ok(Purpose::Service),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateCredentialRequest {
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
        if self.comment.is_some() {
            len += 1;
        }
        if self.read_only.is_some() {
            len += 1;
        }
        if self.owner.is_some() {
            len += 1;
        }
        if self.skip_validation.is_some() {
            len += 1;
        }
        if self.force.is_some() {
            len += 1;
        }
        if self.azure_service_principal.is_some() {
            len += 1;
        }
        if self.azure_managed_identity.is_some() {
            len += 1;
        }
        if self.azure_storage_key.is_some() {
            len += 1;
        }
        if self.aws_iam_role.is_some() {
            len += 1;
        }
        if self.databricks_gcp_service_account.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.credentials.v1.UpdateCredentialRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.new_name.as_ref() {
            struct_ser.serialize_field("new_name", v)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
        }
        if let Some(v) = self.read_only.as_ref() {
            struct_ser.serialize_field("read_only", v)?;
        }
        if let Some(v) = self.owner.as_ref() {
            struct_ser.serialize_field("owner", v)?;
        }
        if let Some(v) = self.skip_validation.as_ref() {
            struct_ser.serialize_field("skip_validation", v)?;
        }
        if let Some(v) = self.force.as_ref() {
            struct_ser.serialize_field("force", v)?;
        }
        if let Some(v) = self.azure_service_principal.as_ref() {
            struct_ser.serialize_field("azure_service_principal", v)?;
        }
        if let Some(v) = self.azure_managed_identity.as_ref() {
            struct_ser.serialize_field("azure_managed_identity", v)?;
        }
        if let Some(v) = self.azure_storage_key.as_ref() {
            struct_ser.serialize_field("azure_storage_key", v)?;
        }
        if let Some(v) = self.aws_iam_role.as_ref() {
            struct_ser.serialize_field("aws_iam_role", v)?;
        }
        if let Some(v) = self.databricks_gcp_service_account.as_ref() {
            struct_ser.serialize_field("databricks_gcp_service_account", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateCredentialRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "new_name",
            "newName",
            "comment",
            "read_only",
            "readOnly",
            "owner",
            "skip_validation",
            "skipValidation",
            "force",
            "azure_service_principal",
            "azureServicePrincipal",
            "azure_managed_identity",
            "azureManagedIdentity",
            "azure_storage_key",
            "azureStorageKey",
            "aws_iam_role",
            "awsIamRole",
            "databricks_gcp_service_account",
            "databricksGcpServiceAccount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            NewName,
            Comment,
            ReadOnly,
            Owner,
            SkipValidation,
            Force,
            AzureServicePrincipal,
            AzureManagedIdentity,
            AzureStorageKey,
            AwsIamRole,
            DatabricksGcpServiceAccount,
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
                            "comment" => Ok(GeneratedField::Comment),
                            "readOnly" | "read_only" => Ok(GeneratedField::ReadOnly),
                            "owner" => Ok(GeneratedField::Owner),
                            "skipValidation" | "skip_validation" => Ok(GeneratedField::SkipValidation),
                            "force" => Ok(GeneratedField::Force),
                            "azureServicePrincipal" | "azure_service_principal" => Ok(GeneratedField::AzureServicePrincipal),
                            "azureManagedIdentity" | "azure_managed_identity" => Ok(GeneratedField::AzureManagedIdentity),
                            "azureStorageKey" | "azure_storage_key" => Ok(GeneratedField::AzureStorageKey),
                            "awsIamRole" | "aws_iam_role" => Ok(GeneratedField::AwsIamRole),
                            "databricksGcpServiceAccount" | "databricks_gcp_service_account" => Ok(GeneratedField::DatabricksGcpServiceAccount),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = UpdateCredentialRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.credentials.v1.UpdateCredentialRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateCredentialRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut new_name__ = None;
                let mut comment__ = None;
                let mut read_only__ = None;
                let mut owner__ = None;
                let mut skip_validation__ = None;
                let mut force__ = None;
                let mut azure_service_principal__ = None;
                let mut azure_managed_identity__ = None;
                let mut azure_storage_key__ = None;
                let mut aws_iam_role__ = None;
                let mut databricks_gcp_service_account__ = None;
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
                        GeneratedField::Comment => {
                            if comment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("comment"));
                            }
                            comment__ = map_.next_value()?;
                        }
                        GeneratedField::ReadOnly => {
                            if read_only__.is_some() {
                                return Err(serde::de::Error::duplicate_field("readOnly"));
                            }
                            read_only__ = map_.next_value()?;
                        }
                        GeneratedField::Owner => {
                            if owner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("owner"));
                            }
                            owner__ = map_.next_value()?;
                        }
                        GeneratedField::SkipValidation => {
                            if skip_validation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("skipValidation"));
                            }
                            skip_validation__ = map_.next_value()?;
                        }
                        GeneratedField::Force => {
                            if force__.is_some() {
                                return Err(serde::de::Error::duplicate_field("force"));
                            }
                            force__ = map_.next_value()?;
                        }
                        GeneratedField::AzureServicePrincipal => {
                            if azure_service_principal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureServicePrincipal"));
                            }
                            azure_service_principal__ = map_.next_value()?;
                        }
                        GeneratedField::AzureManagedIdentity => {
                            if azure_managed_identity__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureManagedIdentity"));
                            }
                            azure_managed_identity__ = map_.next_value()?;
                        }
                        GeneratedField::AzureStorageKey => {
                            if azure_storage_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("azureStorageKey"));
                            }
                            azure_storage_key__ = map_.next_value()?;
                        }
                        GeneratedField::AwsIamRole => {
                            if aws_iam_role__.is_some() {
                                return Err(serde::de::Error::duplicate_field("awsIamRole"));
                            }
                            aws_iam_role__ = map_.next_value()?;
                        }
                        GeneratedField::DatabricksGcpServiceAccount => {
                            if databricks_gcp_service_account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("databricksGcpServiceAccount"));
                            }
                            databricks_gcp_service_account__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(UpdateCredentialRequest {
                    name: name__.unwrap_or_default(),
                    new_name: new_name__,
                    comment: comment__,
                    read_only: read_only__,
                    owner: owner__,
                    skip_validation: skip_validation__,
                    force: force__,
                    azure_service_principal: azure_service_principal__,
                    azure_managed_identity: azure_managed_identity__,
                    azure_storage_key: azure_storage_key__,
                    aws_iam_role: aws_iam_role__,
                    databricks_gcp_service_account: databricks_gcp_service_account__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.credentials.v1.UpdateCredentialRequest", FIELDS, GeneratedVisitor)
    }
}
