// @generated
impl serde::Serialize for CreateFunctionRequest {
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
        if !self.data_type.is_empty() {
            len += 1;
        }
        if !self.full_data_type.is_empty() {
            len += 1;
        }
        if self.input_params.is_some() {
            len += 1;
        }
        if self.parameter_style != 0 {
            len += 1;
        }
        if self.is_deterministic {
            len += 1;
        }
        if self.sql_data_access != 0 {
            len += 1;
        }
        if self.is_null_call {
            len += 1;
        }
        if self.security_type != 0 {
            len += 1;
        }
        if self.routine_body != 0 {
            len += 1;
        }
        if self.routine_definition.is_some() {
            len += 1;
        }
        if self.routine_body_language.is_some() {
            len += 1;
        }
        if self.comment.is_some() {
            len += 1;
        }
        if !self.properties.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.functions.v1.CreateFunctionRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.catalog_name.is_empty() {
            struct_ser.serialize_field("catalog_name", &self.catalog_name)?;
        }
        if !self.schema_name.is_empty() {
            struct_ser.serialize_field("schema_name", &self.schema_name)?;
        }
        if !self.data_type.is_empty() {
            struct_ser.serialize_field("data_type", &self.data_type)?;
        }
        if !self.full_data_type.is_empty() {
            struct_ser.serialize_field("full_data_type", &self.full_data_type)?;
        }
        if let Some(v) = self.input_params.as_ref() {
            struct_ser.serialize_field("input_params", v)?;
        }
        if self.parameter_style != 0 {
            let v = ParameterStyle::try_from(self.parameter_style)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.parameter_style)))?;
            struct_ser.serialize_field("parameter_style", &v)?;
        }
        if self.is_deterministic {
            struct_ser.serialize_field("is_deterministic", &self.is_deterministic)?;
        }
        if self.sql_data_access != 0 {
            let v = SqlDataAccess::try_from(self.sql_data_access)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.sql_data_access)))?;
            struct_ser.serialize_field("sql_data_access", &v)?;
        }
        if self.is_null_call {
            struct_ser.serialize_field("is_null_call", &self.is_null_call)?;
        }
        if self.security_type != 0 {
            let v = SecurityType::try_from(self.security_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.security_type)))?;
            struct_ser.serialize_field("security_type", &v)?;
        }
        if self.routine_body != 0 {
            let v = RoutineBody::try_from(self.routine_body)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.routine_body)))?;
            struct_ser.serialize_field("routine_body", &v)?;
        }
        if let Some(v) = self.routine_definition.as_ref() {
            struct_ser.serialize_field("routine_definition", v)?;
        }
        if let Some(v) = self.routine_body_language.as_ref() {
            struct_ser.serialize_field("routine_body_language", v)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
        }
        if !self.properties.is_empty() {
            struct_ser.serialize_field("properties", &self.properties)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CreateFunctionRequest {
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
            "data_type",
            "dataType",
            "full_data_type",
            "fullDataType",
            "input_params",
            "inputParams",
            "parameter_style",
            "parameterStyle",
            "is_deterministic",
            "isDeterministic",
            "sql_data_access",
            "sqlDataAccess",
            "is_null_call",
            "isNullCall",
            "security_type",
            "securityType",
            "routine_body",
            "routineBody",
            "routine_definition",
            "routineDefinition",
            "routine_body_language",
            "routineBodyLanguage",
            "comment",
            "properties",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            CatalogName,
            SchemaName,
            DataType,
            FullDataType,
            InputParams,
            ParameterStyle,
            IsDeterministic,
            SqlDataAccess,
            IsNullCall,
            SecurityType,
            RoutineBody,
            RoutineDefinition,
            RoutineBodyLanguage,
            Comment,
            Properties,
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
                            "dataType" | "data_type" => Ok(GeneratedField::DataType),
                            "fullDataType" | "full_data_type" => Ok(GeneratedField::FullDataType),
                            "inputParams" | "input_params" => Ok(GeneratedField::InputParams),
                            "parameterStyle" | "parameter_style" => Ok(GeneratedField::ParameterStyle),
                            "isDeterministic" | "is_deterministic" => Ok(GeneratedField::IsDeterministic),
                            "sqlDataAccess" | "sql_data_access" => Ok(GeneratedField::SqlDataAccess),
                            "isNullCall" | "is_null_call" => Ok(GeneratedField::IsNullCall),
                            "securityType" | "security_type" => Ok(GeneratedField::SecurityType),
                            "routineBody" | "routine_body" => Ok(GeneratedField::RoutineBody),
                            "routineDefinition" | "routine_definition" => Ok(GeneratedField::RoutineDefinition),
                            "routineBodyLanguage" | "routine_body_language" => Ok(GeneratedField::RoutineBodyLanguage),
                            "comment" => Ok(GeneratedField::Comment),
                            "properties" => Ok(GeneratedField::Properties),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CreateFunctionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.functions.v1.CreateFunctionRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<CreateFunctionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut catalog_name__ = None;
                let mut schema_name__ = None;
                let mut data_type__ = None;
                let mut full_data_type__ = None;
                let mut input_params__ = None;
                let mut parameter_style__ = None;
                let mut is_deterministic__ = None;
                let mut sql_data_access__ = None;
                let mut is_null_call__ = None;
                let mut security_type__ = None;
                let mut routine_body__ = None;
                let mut routine_definition__ = None;
                let mut routine_body_language__ = None;
                let mut comment__ = None;
                let mut properties__ = None;
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
                        GeneratedField::DataType => {
                            if data_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataType"));
                            }
                            data_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullDataType => {
                            if full_data_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullDataType"));
                            }
                            full_data_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::InputParams => {
                            if input_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inputParams"));
                            }
                            input_params__ = map_.next_value()?;
                        }
                        GeneratedField::ParameterStyle => {
                            if parameter_style__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameterStyle"));
                            }
                            parameter_style__ = Some(map_.next_value::<ParameterStyle>()? as i32);
                        }
                        GeneratedField::IsDeterministic => {
                            if is_deterministic__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isDeterministic"));
                            }
                            is_deterministic__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SqlDataAccess => {
                            if sql_data_access__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sqlDataAccess"));
                            }
                            sql_data_access__ = Some(map_.next_value::<SqlDataAccess>()? as i32);
                        }
                        GeneratedField::IsNullCall => {
                            if is_null_call__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isNullCall"));
                            }
                            is_null_call__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SecurityType => {
                            if security_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("securityType"));
                            }
                            security_type__ = Some(map_.next_value::<SecurityType>()? as i32);
                        }
                        GeneratedField::RoutineBody => {
                            if routine_body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("routineBody"));
                            }
                            routine_body__ = Some(map_.next_value::<RoutineBody>()? as i32);
                        }
                        GeneratedField::RoutineDefinition => {
                            if routine_definition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("routineDefinition"));
                            }
                            routine_definition__ = map_.next_value()?;
                        }
                        GeneratedField::RoutineBodyLanguage => {
                            if routine_body_language__.is_some() {
                                return Err(serde::de::Error::duplicate_field("routineBodyLanguage"));
                            }
                            routine_body_language__ = map_.next_value()?;
                        }
                        GeneratedField::Comment => {
                            if comment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("comment"));
                            }
                            comment__ = map_.next_value()?;
                        }
                        GeneratedField::Properties => {
                            if properties__.is_some() {
                                return Err(serde::de::Error::duplicate_field("properties"));
                            }
                            properties__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(CreateFunctionRequest {
                    name: name__.unwrap_or_default(),
                    catalog_name: catalog_name__.unwrap_or_default(),
                    schema_name: schema_name__.unwrap_or_default(),
                    data_type: data_type__.unwrap_or_default(),
                    full_data_type: full_data_type__.unwrap_or_default(),
                    input_params: input_params__,
                    parameter_style: parameter_style__.unwrap_or_default(),
                    is_deterministic: is_deterministic__.unwrap_or_default(),
                    sql_data_access: sql_data_access__.unwrap_or_default(),
                    is_null_call: is_null_call__.unwrap_or_default(),
                    security_type: security_type__.unwrap_or_default(),
                    routine_body: routine_body__.unwrap_or_default(),
                    routine_definition: routine_definition__,
                    routine_body_language: routine_body_language__,
                    comment: comment__,
                    properties: properties__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.functions.v1.CreateFunctionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for DeleteFunctionRequest {
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
        if self.force.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.functions.v1.DeleteFunctionRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.force.as_ref() {
            struct_ser.serialize_field("force", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DeleteFunctionRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "force",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Force,
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
                            "force" => Ok(GeneratedField::Force),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DeleteFunctionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.functions.v1.DeleteFunctionRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<DeleteFunctionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut force__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Force => {
                            if force__.is_some() {
                                return Err(serde::de::Error::duplicate_field("force"));
                            }
                            force__ = map_.next_value()?;
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(DeleteFunctionRequest {
                    name: name__.unwrap_or_default(),
                    force: force__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.functions.v1.DeleteFunctionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Function {
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
        if !self.data_type.is_empty() {
            len += 1;
        }
        if !self.full_data_type.is_empty() {
            len += 1;
        }
        if self.input_params.is_some() {
            len += 1;
        }
        if self.return_params.is_some() {
            len += 1;
        }
        if self.routine_body_language.is_some() {
            len += 1;
        }
        if self.routine_definition.is_some() {
            len += 1;
        }
        if self.routine_dependencies.is_some() {
            len += 1;
        }
        if self.parameter_style != 0 {
            len += 1;
        }
        if self.is_deterministic {
            len += 1;
        }
        if self.sql_data_access != 0 {
            len += 1;
        }
        if self.is_null_call {
            len += 1;
        }
        if self.security_type != 0 {
            len += 1;
        }
        if self.specific_name.is_some() {
            len += 1;
        }
        if self.routine_body != 0 {
            len += 1;
        }
        if self.comment.is_some() {
            len += 1;
        }
        if !self.properties.is_empty() {
            len += 1;
        }
        if self.owner.is_some() {
            len += 1;
        }
        if self.function_id.is_some() {
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
        let mut struct_ser = serializer.serialize_struct("unitycatalog.functions.v1.Function", len)?;
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
        if !self.data_type.is_empty() {
            struct_ser.serialize_field("data_type", &self.data_type)?;
        }
        if !self.full_data_type.is_empty() {
            struct_ser.serialize_field("full_data_type", &self.full_data_type)?;
        }
        if let Some(v) = self.input_params.as_ref() {
            struct_ser.serialize_field("input_params", v)?;
        }
        if let Some(v) = self.return_params.as_ref() {
            struct_ser.serialize_field("return_params", v)?;
        }
        if let Some(v) = self.routine_body_language.as_ref() {
            struct_ser.serialize_field("routine_body_language", v)?;
        }
        if let Some(v) = self.routine_definition.as_ref() {
            struct_ser.serialize_field("routine_definition", v)?;
        }
        if let Some(v) = self.routine_dependencies.as_ref() {
            struct_ser.serialize_field("routine_dependencies", v)?;
        }
        if self.parameter_style != 0 {
            let v = ParameterStyle::try_from(self.parameter_style)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.parameter_style)))?;
            struct_ser.serialize_field("parameter_style", &v)?;
        }
        if self.is_deterministic {
            struct_ser.serialize_field("is_deterministic", &self.is_deterministic)?;
        }
        if self.sql_data_access != 0 {
            let v = SqlDataAccess::try_from(self.sql_data_access)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.sql_data_access)))?;
            struct_ser.serialize_field("sql_data_access", &v)?;
        }
        if self.is_null_call {
            struct_ser.serialize_field("is_null_call", &self.is_null_call)?;
        }
        if self.security_type != 0 {
            let v = SecurityType::try_from(self.security_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.security_type)))?;
            struct_ser.serialize_field("security_type", &v)?;
        }
        if let Some(v) = self.specific_name.as_ref() {
            struct_ser.serialize_field("specific_name", v)?;
        }
        if self.routine_body != 0 {
            let v = RoutineBody::try_from(self.routine_body)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.routine_body)))?;
            struct_ser.serialize_field("routine_body", &v)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
        }
        if !self.properties.is_empty() {
            struct_ser.serialize_field("properties", &self.properties)?;
        }
        if let Some(v) = self.owner.as_ref() {
            struct_ser.serialize_field("owner", v)?;
        }
        if let Some(v) = self.function_id.as_ref() {
            struct_ser.serialize_field("function_id", v)?;
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
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Function {
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
            "data_type",
            "dataType",
            "full_data_type",
            "fullDataType",
            "input_params",
            "inputParams",
            "return_params",
            "returnParams",
            "routine_body_language",
            "routineBodyLanguage",
            "routine_definition",
            "routineDefinition",
            "routine_dependencies",
            "routineDependencies",
            "parameter_style",
            "parameterStyle",
            "is_deterministic",
            "isDeterministic",
            "sql_data_access",
            "sqlDataAccess",
            "is_null_call",
            "isNullCall",
            "security_type",
            "securityType",
            "specific_name",
            "specificName",
            "routine_body",
            "routineBody",
            "comment",
            "properties",
            "owner",
            "function_id",
            "functionId",
            "created_at",
            "createdAt",
            "created_by",
            "createdBy",
            "updated_at",
            "updatedAt",
            "updated_by",
            "updatedBy",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            CatalogName,
            SchemaName,
            FullName,
            DataType,
            FullDataType,
            InputParams,
            ReturnParams,
            RoutineBodyLanguage,
            RoutineDefinition,
            RoutineDependencies,
            ParameterStyle,
            IsDeterministic,
            SqlDataAccess,
            IsNullCall,
            SecurityType,
            SpecificName,
            RoutineBody,
            Comment,
            Properties,
            Owner,
            FunctionId,
            CreatedAt,
            CreatedBy,
            UpdatedAt,
            UpdatedBy,
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
                            "dataType" | "data_type" => Ok(GeneratedField::DataType),
                            "fullDataType" | "full_data_type" => Ok(GeneratedField::FullDataType),
                            "inputParams" | "input_params" => Ok(GeneratedField::InputParams),
                            "returnParams" | "return_params" => Ok(GeneratedField::ReturnParams),
                            "routineBodyLanguage" | "routine_body_language" => Ok(GeneratedField::RoutineBodyLanguage),
                            "routineDefinition" | "routine_definition" => Ok(GeneratedField::RoutineDefinition),
                            "routineDependencies" | "routine_dependencies" => Ok(GeneratedField::RoutineDependencies),
                            "parameterStyle" | "parameter_style" => Ok(GeneratedField::ParameterStyle),
                            "isDeterministic" | "is_deterministic" => Ok(GeneratedField::IsDeterministic),
                            "sqlDataAccess" | "sql_data_access" => Ok(GeneratedField::SqlDataAccess),
                            "isNullCall" | "is_null_call" => Ok(GeneratedField::IsNullCall),
                            "securityType" | "security_type" => Ok(GeneratedField::SecurityType),
                            "specificName" | "specific_name" => Ok(GeneratedField::SpecificName),
                            "routineBody" | "routine_body" => Ok(GeneratedField::RoutineBody),
                            "comment" => Ok(GeneratedField::Comment),
                            "properties" => Ok(GeneratedField::Properties),
                            "owner" => Ok(GeneratedField::Owner),
                            "functionId" | "function_id" => Ok(GeneratedField::FunctionId),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            "createdBy" | "created_by" => Ok(GeneratedField::CreatedBy),
                            "updatedAt" | "updated_at" => Ok(GeneratedField::UpdatedAt),
                            "updatedBy" | "updated_by" => Ok(GeneratedField::UpdatedBy),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Function;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.functions.v1.Function")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<Function, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut catalog_name__ = None;
                let mut schema_name__ = None;
                let mut full_name__ = None;
                let mut data_type__ = None;
                let mut full_data_type__ = None;
                let mut input_params__ = None;
                let mut return_params__ = None;
                let mut routine_body_language__ = None;
                let mut routine_definition__ = None;
                let mut routine_dependencies__ = None;
                let mut parameter_style__ = None;
                let mut is_deterministic__ = None;
                let mut sql_data_access__ = None;
                let mut is_null_call__ = None;
                let mut security_type__ = None;
                let mut specific_name__ = None;
                let mut routine_body__ = None;
                let mut comment__ = None;
                let mut properties__ = None;
                let mut owner__ = None;
                let mut function_id__ = None;
                let mut created_at__ = None;
                let mut created_by__ = None;
                let mut updated_at__ = None;
                let mut updated_by__ = None;
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
                        GeneratedField::DataType => {
                            if data_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("dataType"));
                            }
                            data_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::FullDataType => {
                            if full_data_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullDataType"));
                            }
                            full_data_type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::InputParams => {
                            if input_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("inputParams"));
                            }
                            input_params__ = map_.next_value()?;
                        }
                        GeneratedField::ReturnParams => {
                            if return_params__.is_some() {
                                return Err(serde::de::Error::duplicate_field("returnParams"));
                            }
                            return_params__ = map_.next_value()?;
                        }
                        GeneratedField::RoutineBodyLanguage => {
                            if routine_body_language__.is_some() {
                                return Err(serde::de::Error::duplicate_field("routineBodyLanguage"));
                            }
                            routine_body_language__ = map_.next_value()?;
                        }
                        GeneratedField::RoutineDefinition => {
                            if routine_definition__.is_some() {
                                return Err(serde::de::Error::duplicate_field("routineDefinition"));
                            }
                            routine_definition__ = map_.next_value()?;
                        }
                        GeneratedField::RoutineDependencies => {
                            if routine_dependencies__.is_some() {
                                return Err(serde::de::Error::duplicate_field("routineDependencies"));
                            }
                            routine_dependencies__ = map_.next_value()?;
                        }
                        GeneratedField::ParameterStyle => {
                            if parameter_style__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameterStyle"));
                            }
                            parameter_style__ = Some(map_.next_value::<ParameterStyle>()? as i32);
                        }
                        GeneratedField::IsDeterministic => {
                            if is_deterministic__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isDeterministic"));
                            }
                            is_deterministic__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SqlDataAccess => {
                            if sql_data_access__.is_some() {
                                return Err(serde::de::Error::duplicate_field("sqlDataAccess"));
                            }
                            sql_data_access__ = Some(map_.next_value::<SqlDataAccess>()? as i32);
                        }
                        GeneratedField::IsNullCall => {
                            if is_null_call__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isNullCall"));
                            }
                            is_null_call__ = Some(map_.next_value()?);
                        }
                        GeneratedField::SecurityType => {
                            if security_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("securityType"));
                            }
                            security_type__ = Some(map_.next_value::<SecurityType>()? as i32);
                        }
                        GeneratedField::SpecificName => {
                            if specific_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("specificName"));
                            }
                            specific_name__ = map_.next_value()?;
                        }
                        GeneratedField::RoutineBody => {
                            if routine_body__.is_some() {
                                return Err(serde::de::Error::duplicate_field("routineBody"));
                            }
                            routine_body__ = Some(map_.next_value::<RoutineBody>()? as i32);
                        }
                        GeneratedField::Comment => {
                            if comment__.is_some() {
                                return Err(serde::de::Error::duplicate_field("comment"));
                            }
                            comment__ = map_.next_value()?;
                        }
                        GeneratedField::Properties => {
                            if properties__.is_some() {
                                return Err(serde::de::Error::duplicate_field("properties"));
                            }
                            properties__ = Some(
                                map_.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::Owner => {
                            if owner__.is_some() {
                                return Err(serde::de::Error::duplicate_field("owner"));
                            }
                            owner__ = map_.next_value()?;
                        }
                        GeneratedField::FunctionId => {
                            if function_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("functionId"));
                            }
                            function_id__ = map_.next_value()?;
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
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(Function {
                    name: name__.unwrap_or_default(),
                    catalog_name: catalog_name__.unwrap_or_default(),
                    schema_name: schema_name__.unwrap_or_default(),
                    full_name: full_name__.unwrap_or_default(),
                    data_type: data_type__.unwrap_or_default(),
                    full_data_type: full_data_type__.unwrap_or_default(),
                    input_params: input_params__,
                    return_params: return_params__,
                    routine_body_language: routine_body_language__,
                    routine_definition: routine_definition__,
                    routine_dependencies: routine_dependencies__,
                    parameter_style: parameter_style__.unwrap_or_default(),
                    is_deterministic: is_deterministic__.unwrap_or_default(),
                    sql_data_access: sql_data_access__.unwrap_or_default(),
                    is_null_call: is_null_call__.unwrap_or_default(),
                    security_type: security_type__.unwrap_or_default(),
                    specific_name: specific_name__,
                    routine_body: routine_body__.unwrap_or_default(),
                    comment: comment__,
                    properties: properties__.unwrap_or_default(),
                    owner: owner__,
                    function_id: function_id__,
                    created_at: created_at__,
                    created_by: created_by__,
                    updated_at: updated_at__,
                    updated_by: updated_by__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.functions.v1.Function", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FunctionParameterInfo {
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
        if !self.type_text.is_empty() {
            len += 1;
        }
        if self.type_json.is_some() {
            len += 1;
        }
        if self.type_name != 0 {
            len += 1;
        }
        if self.type_precision.is_some() {
            len += 1;
        }
        if self.type_scale.is_some() {
            len += 1;
        }
        if self.type_interval_type.is_some() {
            len += 1;
        }
        if self.position.is_some() {
            len += 1;
        }
        if self.parameter_mode != 0 {
            len += 1;
        }
        if self.parameter_type != 0 {
            len += 1;
        }
        if self.parameter_default.is_some() {
            len += 1;
        }
        if self.comment.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.functions.v1.FunctionParameterInfo", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.type_text.is_empty() {
            struct_ser.serialize_field("type_text", &self.type_text)?;
        }
        if let Some(v) = self.type_json.as_ref() {
            struct_ser.serialize_field("type_json", v)?;
        }
        if self.type_name != 0 {
            let v = super::super::tables::v1::ColumnTypeName::try_from(self.type_name)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.type_name)))?;
            struct_ser.serialize_field("type_name", &v)?;
        }
        if let Some(v) = self.type_precision.as_ref() {
            struct_ser.serialize_field("type_precision", v)?;
        }
        if let Some(v) = self.type_scale.as_ref() {
            struct_ser.serialize_field("type_scale", v)?;
        }
        if let Some(v) = self.type_interval_type.as_ref() {
            struct_ser.serialize_field("type_interval_type", v)?;
        }
        if let Some(v) = self.position.as_ref() {
            struct_ser.serialize_field("position", v)?;
        }
        if self.parameter_mode != 0 {
            let v = ParameterMode::try_from(self.parameter_mode)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.parameter_mode)))?;
            struct_ser.serialize_field("parameter_mode", &v)?;
        }
        if self.parameter_type != 0 {
            let v = FunctionParameterType::try_from(self.parameter_type)
                .map_err(|_| serde::ser::Error::custom(format!("Invalid variant {}", self.parameter_type)))?;
            struct_ser.serialize_field("parameter_type", &v)?;
        }
        if let Some(v) = self.parameter_default.as_ref() {
            struct_ser.serialize_field("parameter_default", v)?;
        }
        if let Some(v) = self.comment.as_ref() {
            struct_ser.serialize_field("comment", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FunctionParameterInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "type_text",
            "typeText",
            "type_json",
            "typeJson",
            "type_name",
            "typeName",
            "type_precision",
            "typePrecision",
            "type_scale",
            "typeScale",
            "type_interval_type",
            "typeIntervalType",
            "position",
            "parameter_mode",
            "parameterMode",
            "parameter_type",
            "parameterType",
            "parameter_default",
            "parameterDefault",
            "comment",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            TypeText,
            TypeJson,
            TypeName,
            TypePrecision,
            TypeScale,
            TypeIntervalType,
            Position,
            ParameterMode,
            ParameterType,
            ParameterDefault,
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
                            "name" => Ok(GeneratedField::Name),
                            "typeText" | "type_text" => Ok(GeneratedField::TypeText),
                            "typeJson" | "type_json" => Ok(GeneratedField::TypeJson),
                            "typeName" | "type_name" => Ok(GeneratedField::TypeName),
                            "typePrecision" | "type_precision" => Ok(GeneratedField::TypePrecision),
                            "typeScale" | "type_scale" => Ok(GeneratedField::TypeScale),
                            "typeIntervalType" | "type_interval_type" => Ok(GeneratedField::TypeIntervalType),
                            "position" => Ok(GeneratedField::Position),
                            "parameterMode" | "parameter_mode" => Ok(GeneratedField::ParameterMode),
                            "parameterType" | "parameter_type" => Ok(GeneratedField::ParameterType),
                            "parameterDefault" | "parameter_default" => Ok(GeneratedField::ParameterDefault),
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
            type Value = FunctionParameterInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.functions.v1.FunctionParameterInfo")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FunctionParameterInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut type_text__ = None;
                let mut type_json__ = None;
                let mut type_name__ = None;
                let mut type_precision__ = None;
                let mut type_scale__ = None;
                let mut type_interval_type__ = None;
                let mut position__ = None;
                let mut parameter_mode__ = None;
                let mut parameter_type__ = None;
                let mut parameter_default__ = None;
                let mut comment__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TypeText => {
                            if type_text__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeText"));
                            }
                            type_text__ = Some(map_.next_value()?);
                        }
                        GeneratedField::TypeJson => {
                            if type_json__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeJson"));
                            }
                            type_json__ = map_.next_value()?;
                        }
                        GeneratedField::TypeName => {
                            if type_name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeName"));
                            }
                            type_name__ = Some(map_.next_value::<super::super::tables::v1::ColumnTypeName>()? as i32);
                        }
                        GeneratedField::TypePrecision => {
                            if type_precision__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typePrecision"));
                            }
                            type_precision__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::TypeScale => {
                            if type_scale__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeScale"));
                            }
                            type_scale__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::TypeIntervalType => {
                            if type_interval_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeIntervalType"));
                            }
                            type_interval_type__ = map_.next_value()?;
                        }
                        GeneratedField::Position => {
                            if position__.is_some() {
                                return Err(serde::de::Error::duplicate_field("position"));
                            }
                            position__ = 
                                map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| x.0)
                            ;
                        }
                        GeneratedField::ParameterMode => {
                            if parameter_mode__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameterMode"));
                            }
                            parameter_mode__ = Some(map_.next_value::<ParameterMode>()? as i32);
                        }
                        GeneratedField::ParameterType => {
                            if parameter_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameterType"));
                            }
                            parameter_type__ = Some(map_.next_value::<FunctionParameterType>()? as i32);
                        }
                        GeneratedField::ParameterDefault => {
                            if parameter_default__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameterDefault"));
                            }
                            parameter_default__ = map_.next_value()?;
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
                Ok(FunctionParameterInfo {
                    name: name__.unwrap_or_default(),
                    type_text: type_text__.unwrap_or_default(),
                    type_json: type_json__,
                    type_name: type_name__.unwrap_or_default(),
                    type_precision: type_precision__,
                    type_scale: type_scale__,
                    type_interval_type: type_interval_type__,
                    position: position__,
                    parameter_mode: parameter_mode__.unwrap_or_default(),
                    parameter_type: parameter_type__.unwrap_or_default(),
                    parameter_default: parameter_default__,
                    comment: comment__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.functions.v1.FunctionParameterInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FunctionParameterInfos {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.parameters.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.functions.v1.FunctionParameterInfos", len)?;
        if !self.parameters.is_empty() {
            struct_ser.serialize_field("parameters", &self.parameters)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FunctionParameterInfos {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "parameters",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Parameters,
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
                            "parameters" => Ok(GeneratedField::Parameters),
                            _ => Ok(GeneratedField::__SkipField__),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FunctionParameterInfos;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.functions.v1.FunctionParameterInfos")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<FunctionParameterInfos, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut parameters__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Parameters => {
                            if parameters__.is_some() {
                                return Err(serde::de::Error::duplicate_field("parameters"));
                            }
                            parameters__ = Some(map_.next_value()?);
                        }
                        GeneratedField::__SkipField__ => {
                            let _ = map_.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }
                Ok(FunctionParameterInfos {
                    parameters: parameters__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.functions.v1.FunctionParameterInfos", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FunctionParameterType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "FUNCTION_PARAMETER_TYPE_UNSPECIFIED",
            Self::Column => "COLUMN",
            Self::Param => "PARAM",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for FunctionParameterType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "FUNCTION_PARAMETER_TYPE_UNSPECIFIED",
            "COLUMN",
            "PARAM",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FunctionParameterType;

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
                    "FUNCTION_PARAMETER_TYPE_UNSPECIFIED" => Ok(FunctionParameterType::Unspecified),
                    "COLUMN" => Ok(FunctionParameterType::Column),
                    "PARAM" => Ok(FunctionParameterType::Param),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for GetFunctionRequest {
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
        let mut struct_ser = serializer.serialize_struct("unitycatalog.functions.v1.GetFunctionRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetFunctionRequest {
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
            type Value = GetFunctionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.functions.v1.GetFunctionRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<GetFunctionRequest, V::Error>
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
                Ok(GetFunctionRequest {
                    name: name__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.functions.v1.GetFunctionRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListFunctionsRequest {
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
        let mut struct_ser = serializer.serialize_struct("unitycatalog.functions.v1.ListFunctionsRequest", len)?;
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
impl<'de> serde::Deserialize<'de> for ListFunctionsRequest {
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
            type Value = ListFunctionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.functions.v1.ListFunctionsRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListFunctionsRequest, V::Error>
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
                Ok(ListFunctionsRequest {
                    catalog_name: catalog_name__.unwrap_or_default(),
                    schema_name: schema_name__.unwrap_or_default(),
                    max_results: max_results__,
                    page_token: page_token__,
                    include_browse: include_browse__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.functions.v1.ListFunctionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ListFunctionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.functions.is_empty() {
            len += 1;
        }
        if self.next_page_token.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.functions.v1.ListFunctionsResponse", len)?;
        if !self.functions.is_empty() {
            struct_ser.serialize_field("functions", &self.functions)?;
        }
        if let Some(v) = self.next_page_token.as_ref() {
            struct_ser.serialize_field("next_page_token", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ListFunctionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "functions",
            "next_page_token",
            "nextPageToken",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Functions,
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
                            "functions" => Ok(GeneratedField::Functions),
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
            type Value = ListFunctionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.functions.v1.ListFunctionsResponse")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ListFunctionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut functions__ = None;
                let mut next_page_token__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Functions => {
                            if functions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("functions"));
                            }
                            functions__ = Some(map_.next_value()?);
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
                Ok(ListFunctionsResponse {
                    functions: functions__.unwrap_or_default(),
                    next_page_token: next_page_token__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.functions.v1.ListFunctionsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ParameterMode {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "PARAMETER_MODE_UNSPECIFIED",
            Self::In => "IN",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ParameterMode {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "PARAMETER_MODE_UNSPECIFIED",
            "IN",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ParameterMode;

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
                    "PARAMETER_MODE_UNSPECIFIED" => Ok(ParameterMode::Unspecified),
                    "IN" => Ok(ParameterMode::In),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ParameterStyle {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "PARAMETER_STYLE_UNSPECIFIED",
            Self::S => "S",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ParameterStyle {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "PARAMETER_STYLE_UNSPECIFIED",
            "S",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ParameterStyle;

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
                    "PARAMETER_STYLE_UNSPECIFIED" => Ok(ParameterStyle::Unspecified),
                    "S" => Ok(ParameterStyle::S),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for RoutineBody {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "ROUTINE_BODY_UNSPECIFIED",
            Self::Sql => "SQL",
            Self::External => "EXTERNAL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for RoutineBody {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ROUTINE_BODY_UNSPECIFIED",
            "SQL",
            "EXTERNAL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RoutineBody;

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
                    "ROUTINE_BODY_UNSPECIFIED" => Ok(RoutineBody::Unspecified),
                    "SQL" => Ok(RoutineBody::Sql),
                    "EXTERNAL" => Ok(RoutineBody::External),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for SecurityType {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SECURITY_TYPE_UNSPECIFIED",
            Self::Definer => "DEFINER",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for SecurityType {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SECURITY_TYPE_UNSPECIFIED",
            "DEFINER",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SecurityType;

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
                    "SECURITY_TYPE_UNSPECIFIED" => Ok(SecurityType::Unspecified),
                    "DEFINER" => Ok(SecurityType::Definer),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for SqlDataAccess {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "SQL_DATA_ACCESS_UNSPECIFIED",
            Self::ContainsSql => "CONTAINS_SQL",
            Self::ReadsSqlData => "READS_SQL_DATA",
            Self::NoSql => "NO_SQL",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for SqlDataAccess {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "SQL_DATA_ACCESS_UNSPECIFIED",
            "CONTAINS_SQL",
            "READS_SQL_DATA",
            "NO_SQL",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SqlDataAccess;

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
                    "SQL_DATA_ACCESS_UNSPECIFIED" => Ok(SqlDataAccess::Unspecified),
                    "CONTAINS_SQL" => Ok(SqlDataAccess::ContainsSql),
                    "READS_SQL_DATA" => Ok(SqlDataAccess::ReadsSqlData),
                    "NO_SQL" => Ok(SqlDataAccess::NoSql),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for UpdateFunctionRequest {
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
        if self.owner.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("unitycatalog.functions.v1.UpdateFunctionRequest", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.owner.as_ref() {
            struct_ser.serialize_field("owner", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for UpdateFunctionRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "owner",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
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
            type Value = UpdateFunctionRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct unitycatalog.functions.v1.UpdateFunctionRequest")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<UpdateFunctionRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut owner__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
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
                Ok(UpdateFunctionRequest {
                    name: name__.unwrap_or_default(),
                    owner: owner__,
                })
            }
        }
        deserializer.deserialize_struct("unitycatalog.functions.v1.UpdateFunctionRequest", FIELDS, GeneratedVisitor)
    }
}
