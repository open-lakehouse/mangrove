// @generated
impl serde::Serialize for CreatePolicyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.on_securable_type.is_empty() {
            len += 1;
        }
        if !self.on_securable_fullname.is_empty() {
            len += 1;
        }
        if self.policy_info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.CreatePolicyRequest", len)?;
        if !self.on_securable_type.is_empty() {
            struct_ser.serialize_field("on_securable_type", &self.on_securable_type)?;
        }
        if !self.on_securable_fullname.is_empty() {
            struct_ser.serialize_field("on_securable_fullname", &self.on_securable_fullname)?;
        }
        if let Some(v) = self.policy_info.as_ref() {
            struct_ser.serialize_field("policy_info", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreatePolicyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "on_securable_type",
            "onSecurableType",
            "on_securable_fullname",
            "onSecurableFullname",
            "policy_info",
            "policyInfo",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OnSecurableType,
            OnSecurableFullname,
            PolicyInfo,
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
                            "onSecurableType" | "on_securable_type" => Ok(GeneratedField::OnSecurableType),
                            "onSecurableFullname" | "on_securable_fullname" => Ok(GeneratedField::OnSecurableFullname),
                            "policyInfo" | "policy_info" => Ok(GeneratedField::PolicyInfo),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreatePolicyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.CreatePolicyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreatePolicyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut on_securable_type__ = None;
                let mut on_securable_fullname__ = None;
                let mut policy_info__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OnSecurableType => {
                            if on_securable_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableType"));
                            }
                            on_securable_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OnSecurableFullname => {
                            if on_securable_fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableFullname"));
                            }
                            on_securable_fullname__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PolicyInfo => {
                            if policy_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyInfo"));
                            }
                            policy_info__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CreatePolicyRequest {
                    on_securable_type: on_securable_type__.unwrap_or_default(),
                    on_securable_fullname: on_securable_fullname__.unwrap_or_default(),
                    policy_info: policy_info__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.CreatePolicyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeletePolicyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.on_securable_type.is_empty() {
            len += 1;
        }
        if !self.on_securable_fullname.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.DeletePolicyRequest", len)?;
        if !self.on_securable_type.is_empty() {
            struct_ser.serialize_field("on_securable_type", &self.on_securable_type)?;
        }
        if !self.on_securable_fullname.is_empty() {
            struct_ser.serialize_field("on_securable_fullname", &self.on_securable_fullname)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeletePolicyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "on_securable_type",
            "onSecurableType",
            "on_securable_fullname",
            "onSecurableFullname",
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OnSecurableType,
            OnSecurableFullname,
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
                            "onSecurableType" | "on_securable_type" => Ok(GeneratedField::OnSecurableType),
                            "onSecurableFullname" | "on_securable_fullname" => Ok(GeneratedField::OnSecurableFullname),
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
            type Value = DeletePolicyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.DeletePolicyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeletePolicyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut on_securable_type__ = None;
                let mut on_securable_fullname__ = None;
                let mut name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OnSecurableType => {
                            if on_securable_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableType"));
                            }
                            on_securable_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OnSecurableFullname => {
                            if on_securable_fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableFullname"));
                            }
                            on_securable_fullname__ = Some(map_.next_value()?);
                        }
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
                Ok(DeletePolicyRequest {
                    on_securable_type: on_securable_type__.unwrap_or_default(),
                    on_securable_fullname: on_securable_fullname__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.DeletePolicyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FunctionArg {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.FunctionArg", len)?;
        if let Some(v) = self.value.as_ref() {
            match v {
                function_arg::Value::Alias(v) => {
                    struct_ser.serialize_field("alias", v)?;
                }
                function_arg::Value::Constant(v) => {
                    struct_ser.serialize_field("constant", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FunctionArg {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "alias",
            "constant",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Alias,
            Constant,
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
                            "alias" => Ok(GeneratedField::Alias),
                            "constant" => Ok(GeneratedField::Constant),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FunctionArg;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.FunctionArg")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FunctionArg, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Alias => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("alias"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(function_arg::Value::Alias);
                        }
                        GeneratedField::Constant => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("constant"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(function_arg::Value::Constant);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FunctionArg {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.FunctionArg", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FunctionRef {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.function_name.is_empty() {
            len += 1;
        }
        if !self.using.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.FunctionRef", len)?;
        if !self.function_name.is_empty() {
            struct_ser.serialize_field("function_name", &self.function_name)?;
        }
        if !self.using.is_empty() {
            struct_ser.serialize_field("using", &self.using)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FunctionRef {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "function_name",
            "functionName",
            "using",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            FunctionName,
            Using,
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
                            "functionName" | "function_name" => Ok(GeneratedField::FunctionName),
                            "using" => Ok(GeneratedField::Using),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FunctionRef;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.FunctionRef")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FunctionRef, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut function_name__ = None;
                let mut using__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::FunctionName => {
                            if function_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("functionName"));
                            }
                            function_name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Using => {
                            if using__.is_some() {
                                return Err(serde::de::Error::duplicate_field("using"));
                            }
                            using__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FunctionRef {
                    function_name: function_name__.unwrap_or_default(),
                    using: using__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.FunctionRef", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetPolicyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.on_securable_type.is_empty() {
            len += 1;
        }
        if !self.on_securable_fullname.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.GetPolicyRequest", len)?;
        if !self.on_securable_type.is_empty() {
            struct_ser.serialize_field("on_securable_type", &self.on_securable_type)?;
        }
        if !self.on_securable_fullname.is_empty() {
            struct_ser.serialize_field("on_securable_fullname", &self.on_securable_fullname)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetPolicyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "on_securable_type",
            "onSecurableType",
            "on_securable_fullname",
            "onSecurableFullname",
            "name",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OnSecurableType,
            OnSecurableFullname,
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
                            "onSecurableType" | "on_securable_type" => Ok(GeneratedField::OnSecurableType),
                            "onSecurableFullname" | "on_securable_fullname" => Ok(GeneratedField::OnSecurableFullname),
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
            type Value = GetPolicyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.GetPolicyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetPolicyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut on_securable_type__ = None;
                let mut on_securable_fullname__ = None;
                let mut name__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OnSecurableType => {
                            if on_securable_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableType"));
                            }
                            on_securable_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OnSecurableFullname => {
                            if on_securable_fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableFullname"));
                            }
                            on_securable_fullname__ = Some(map_.next_value()?);
                        }
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
                Ok(GetPolicyRequest {
                    on_securable_type: on_securable_type__.unwrap_or_default(),
                    on_securable_fullname: on_securable_fullname__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.GetPolicyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListPoliciesRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.on_securable_type.is_empty() {
            len += 1;
        }
        if !self.on_securable_fullname.is_empty() {
            len += 1;
        }
        if self.include_inherited.is_some() {
            len += 1;
        }
        if self.max_results.is_some() {
            len += 1;
        }
        if self.page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.ListPoliciesRequest", len)?;
        if !self.on_securable_type.is_empty() {
            struct_ser.serialize_field("on_securable_type", &self.on_securable_type)?;
        }
        if !self.on_securable_fullname.is_empty() {
            struct_ser.serialize_field("on_securable_fullname", &self.on_securable_fullname)?;
        }
        if let Some(v) = self.include_inherited.as_ref() {
            struct_ser.serialize_field("include_inherited", v)?;
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
impl<'de> serde::Deserialize<'de> for ListPoliciesRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "on_securable_type",
            "onSecurableType",
            "on_securable_fullname",
            "onSecurableFullname",
            "include_inherited",
            "includeInherited",
            "max_results",
            "maxResults",
            "page_token",
            "pageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OnSecurableType,
            OnSecurableFullname,
            IncludeInherited,
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
                            "onSecurableType" | "on_securable_type" => Ok(GeneratedField::OnSecurableType),
                            "onSecurableFullname" | "on_securable_fullname" => Ok(GeneratedField::OnSecurableFullname),
                            "includeInherited" | "include_inherited" => Ok(GeneratedField::IncludeInherited),
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
            type Value = ListPoliciesRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.ListPoliciesRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListPoliciesRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut on_securable_type__ = None;
                let mut on_securable_fullname__ = None;
                let mut include_inherited__ = None;
                let mut max_results__ = None;
                let mut page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OnSecurableType => {
                            if on_securable_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableType"));
                            }
                            on_securable_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OnSecurableFullname => {
                            if on_securable_fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableFullname"));
                            }
                            on_securable_fullname__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IncludeInherited => {
                            if include_inherited__.is_some() {
                                return Err(serde::de::Error::duplicate_field("includeInherited"));
                            }
                            include_inherited__ = map_.next_value()?;
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
                Ok(ListPoliciesRequest {
                    on_securable_type: on_securable_type__.unwrap_or_default(),
                    on_securable_fullname: on_securable_fullname__.unwrap_or_default(),
                    include_inherited: include_inherited__,
                    max_results: max_results__,
                    page_token: page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.ListPoliciesRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListPoliciesResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.policies.is_empty() {
            len += 1;
        }
        if self.next_page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.ListPoliciesResponse", len)?;
        if !self.policies.is_empty() {
            struct_ser.serialize_field("policies", &self.policies)?;
        }
        if let Some(v) = self.next_page_token.as_ref() {
            struct_ser.serialize_field("next_page_token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListPoliciesResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "policies",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Policies,
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
                            "policies" => Ok(GeneratedField::Policies),
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
            type Value = ListPoliciesResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.ListPoliciesResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListPoliciesResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut policies__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Policies => {
                            if policies__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policies"));
                            }
                            policies__ = Some(map_.next_value()?);
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
                Ok(ListPoliciesResponse {
                    policies: policies__.unwrap_or_default(),
                    next_page_token: next_page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.ListPoliciesResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MatchColumn {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.alias.is_empty() {
            len += 1;
        }
        if !self.condition.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.MatchColumn", len)?;
        if !self.alias.is_empty() {
            struct_ser.serialize_field("alias", &self.alias)?;
        }
        if !self.condition.is_empty() {
            struct_ser.serialize_field("condition", &self.condition)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MatchColumn {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "alias",
            "condition",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Alias,
            Condition,
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
                            "alias" => Ok(GeneratedField::Alias),
                            "condition" => Ok(GeneratedField::Condition),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MatchColumn;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.MatchColumn")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<MatchColumn, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut alias__ = None;
                let mut condition__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Alias => {
                            if alias__.is_some() {
                                return Err(serde::de::Error::duplicate_field("alias"));
                            }
                            alias__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Condition => {
                            if condition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("condition"));
                            }
                            condition__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(MatchColumn {
                    alias: alias__.unwrap_or_default(),
                    condition: condition__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.MatchColumn", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PolicyInfo {
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
        if !self.on_securable_type.is_empty() {
            len += 1;
        }
        if !self.on_securable_fullname.is_empty() {
            len += 1;
        }
        if self.policy_type != 0 {
            len += 1;
        }
        if !self.to_principals.is_empty() {
            len += 1;
        }
        if !self.except_principals.is_empty() {
            len += 1;
        }
        if self.when_condition.is_some() {
            len += 1;
        }
        if !self.match_columns.is_empty() {
            len += 1;
        }
        if self.comment.is_some() {
            len += 1;
        }
        if self.created_at.is_some() {
            len += 1;
        }
        if self.updated_at.is_some() {
            len += 1;
        }
        if self.function.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.PolicyInfo", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.id.as_ref() {
            struct_ser.serialize_field("id", v)?;
        }
        if !self.on_securable_type.is_empty() {
            struct_ser.serialize_field("on_securable_type", &self.on_securable_type)?;
        }
        if !self.on_securable_fullname.is_empty() {
            struct_ser.serialize_field("on_securable_fullname", &self.on_securable_fullname)?;
        }
        if self.policy_type != 0 {
            let v = PolicyType::try_from(self.policy_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.policy_type)))?;
            struct_ser.serialize_field("policy_type", &v)?;
        }
        if !self.to_principals.is_empty() {
            struct_ser.serialize_field("to_principals", &self.to_principals)?;
        }
        if !self.except_principals.is_empty() {
            struct_ser.serialize_field("except_principals", &self.except_principals)?;
        }
        if let Some(v) = self.when_condition.as_ref() {
            struct_ser.serialize_field("when_condition", v)?;
        }
        if !self.match_columns.is_empty() {
            struct_ser.serialize_field("match_columns", &self.match_columns)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
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
        if let Some(v) = self.function.as_ref() {
            match v {
                policy_info::Function::RowFilter(v) => {
                    struct_ser.serialize_field("row_filter", v)?;
                }
                policy_info::Function::ColumnMask(v) => {
                    struct_ser.serialize_field("column_mask", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PolicyInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "id",
            "on_securable_type",
            "onSecurableType",
            "on_securable_fullname",
            "onSecurableFullname",
            "policy_type",
            "policyType",
            "to_principals",
            "toPrincipals",
            "except_principals",
            "exceptPrincipals",
            "when_condition",
            "whenCondition",
            "match_columns",
            "matchColumns",
            "comment",
            "created_at",
            "createdAt",
            "updated_at",
            "updatedAt",
            "row_filter",
            "rowFilter",
            "column_mask",
            "columnMask",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Id,
            OnSecurableType,
            OnSecurableFullname,
            PolicyType,
            ToPrincipals,
            ExceptPrincipals,
            WhenCondition,
            MatchColumns,
            Comment,
            CreatedAt,
            UpdatedAt,
            RowFilter,
            ColumnMask,
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
                            "onSecurableType" | "on_securable_type" => Ok(GeneratedField::OnSecurableType),
                            "onSecurableFullname" | "on_securable_fullname" => Ok(GeneratedField::OnSecurableFullname),
                            "policyType" | "policy_type" => Ok(GeneratedField::PolicyType),
                            "toPrincipals" | "to_principals" => Ok(GeneratedField::ToPrincipals),
                            "exceptPrincipals" | "except_principals" => Ok(GeneratedField::ExceptPrincipals),
                            "whenCondition" | "when_condition" => Ok(GeneratedField::WhenCondition),
                            "matchColumns" | "match_columns" => Ok(GeneratedField::MatchColumns),
                            "comment" => Ok(GeneratedField::Comment),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            "rowFilter" | "row_filter" => Ok(GeneratedField::RowFilter),
                            "columnMask" | "column_mask" => Ok(GeneratedField::ColumnMask),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PolicyInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.PolicyInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<PolicyInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut id__ = None;
                let mut on_securable_type__ = None;
                let mut on_securable_fullname__ = None;
                let mut policy_type__ = None;
                let mut to_principals__ = None;
                let mut except_principals__ = None;
                let mut when_condition__ = None;
                let mut match_columns__ = None;
                let mut comment__ = None;
                let mut created_at__ = None;
                let mut updated_at__ = None;
                let mut function__ = None;
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
                        GeneratedField::OnSecurableType => {
                            if on_securable_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableType"));
                            }
                            on_securable_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OnSecurableFullname => {
                            if on_securable_fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableFullname"));
                            }
                            on_securable_fullname__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PolicyType => {
                            if policy_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyType"));
                            }
                            policy_type__ = Some(map_.next_value::<PolicyType>()? as i32);
                        }
                        GeneratedField::ToPrincipals => {
                            if to_principals__.is_some() {
                                return Err(serde::de::Error::duplicate_field("toPrincipals"));
                            }
                            to_principals__ = Some(map_.next_value()?);
                        }
                        GeneratedField::ExceptPrincipals => {
                            if except_principals__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exceptPrincipals"));
                            }
                            except_principals__ = Some(map_.next_value()?);
                        }
                        GeneratedField::WhenCondition => {
                            if when_condition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("whenCondition"));
                            }
                            when_condition__ = map_.next_value()?;
                        }
                        GeneratedField::MatchColumns => {
                            if match_columns__.is_some() {
                                return Err(serde::de::Error::duplicate_field("matchColumns"));
                            }
                            match_columns__ = Some(map_.next_value()?);
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
                        GeneratedField::UpdatedAt => {
                            if updated_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("updatedAt"));
                            }
                            updated_at__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::RowFilter => {
                            if function__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rowFilter"));
                            }
                            function__ = map_.next_value::<::std::option::Option<_>>()?.map(policy_info::Function::RowFilter)
;
                        }
                        GeneratedField::ColumnMask => {
                            if function__.is_some() {
                                return Err(serde::de::Error::duplicate_field("columnMask"));
                            }
                            function__ = map_.next_value::<::std::option::Option<_>>()?.map(policy_info::Function::ColumnMask)
;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(PolicyInfo {
                    name: name__.unwrap_or_default(),
                    id: id__,
                    on_securable_type: on_securable_type__.unwrap_or_default(),
                    on_securable_fullname: on_securable_fullname__.unwrap_or_default(),
                    policy_type: policy_type__.unwrap_or_default(),
                    to_principals: to_principals__.unwrap_or_default(),
                    except_principals: except_principals__.unwrap_or_default(),
                    when_condition: when_condition__,
                    match_columns: match_columns__.unwrap_or_default(),
                    comment: comment__,
                    created_at: created_at__,
                    updated_at: updated_at__,
                    function: function__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.PolicyInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PolicyType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "POLICY_TYPE_UNSPECIFIED",
            Self::RowFilter => "POLICY_TYPE_ROW_FILTER",
            Self::ColumnMask => "POLICY_TYPE_COLUMN_MASK",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for PolicyType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "POLICY_TYPE_UNSPECIFIED",
            "POLICY_TYPE_ROW_FILTER",
            "POLICY_TYPE_COLUMN_MASK",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PolicyType;

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
                    "POLICY_TYPE_UNSPECIFIED" => Ok(PolicyType::Unspecified),
                    "POLICY_TYPE_ROW_FILTER" => Ok(PolicyType::RowFilter),
                    "POLICY_TYPE_COLUMN_MASK" => Ok(PolicyType::ColumnMask),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for UpdatePolicyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.on_securable_type.is_empty() {
            len += 1;
        }
        if !self.on_securable_fullname.is_empty() {
            len += 1;
        }
        if !self.name.is_empty() {
            len += 1;
        }
        if self.policy_info.is_some() {
            len += 1;
        }
        if self.update_mask.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.policies.v1.UpdatePolicyRequest", len)?;
        if !self.on_securable_type.is_empty() {
            struct_ser.serialize_field("on_securable_type", &self.on_securable_type)?;
        }
        if !self.on_securable_fullname.is_empty() {
            struct_ser.serialize_field("on_securable_fullname", &self.on_securable_fullname)?;
        }
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.policy_info.as_ref() {
            struct_ser.serialize_field("policy_info", v)?;
        }
        if let Some(v) = self.update_mask.as_ref() {
            struct_ser.serialize_field("update_mask", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdatePolicyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "on_securable_type",
            "onSecurableType",
            "on_securable_fullname",
            "onSecurableFullname",
            "name",
            "policy_info",
            "policyInfo",
            "update_mask",
            "updateMask",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            OnSecurableType,
            OnSecurableFullname,
            Name,
            PolicyInfo,
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
                            "onSecurableType" | "on_securable_type" => Ok(GeneratedField::OnSecurableType),
                            "onSecurableFullname" | "on_securable_fullname" => Ok(GeneratedField::OnSecurableFullname),
                            "name" => Ok(GeneratedField::Name),
                            "policyInfo" | "policy_info" => Ok(GeneratedField::PolicyInfo),
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
            type Value = UpdatePolicyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.policies.v1.UpdatePolicyRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdatePolicyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut on_securable_type__ = None;
                let mut on_securable_fullname__ = None;
                let mut name__ = None;
                let mut policy_info__ = None;
                let mut update_mask__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::OnSecurableType => {
                            if on_securable_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableType"));
                            }
                            on_securable_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::OnSecurableFullname => {
                            if on_securable_fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("onSecurableFullname"));
                            }
                            on_securable_fullname__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::PolicyInfo => {
                            if policy_info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("policyInfo"));
                            }
                            policy_info__ = map_.next_value()?;
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
                Ok(UpdatePolicyRequest {
                    on_securable_type: on_securable_type__.unwrap_or_default(),
                    on_securable_fullname: on_securable_fullname__.unwrap_or_default(),
                    name: name__.unwrap_or_default(),
                    policy_info: policy_info__,
                    update_mask: update_mask__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.policies.v1.UpdatePolicyRequest", FIELDS, GeneratedVisitor)
    }
}
