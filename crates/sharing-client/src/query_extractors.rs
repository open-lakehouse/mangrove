//! Hand-written axum request extractors for the Delta Sharing **query** path
//! (`get_table_version`, `get_table_metadata`, `query_table`).
//!
//! These RPCs are deliberately not part of the generated `SharingService`
//! (their responses are newline-delimited JSON, a streaming contract the code
//! generator does not model — see `proto/sharing/open_sharing/v1/svc.proto`), so
//! their extractors are maintained here rather than generated alongside the
//! discovery extractors in `codegen/extractors`.

use axum::response::IntoResponse;
use axum::{RequestExt, RequestPartsExt};

use crate::Result;
use crate::models::open_sharing::v1::{
    GetTableMetadataRequest, GetTableVersionRequest, QueryTableRequest,
};

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableVersionRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            starting_timestamp: Option<String>,
        }
        let axum_extra::extract::Query(QueryParams { starting_timestamp }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetTableVersionRequest {
            share,
            schema,
            name,
            starting_timestamp,
            ..Default::default()
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableMetadataRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetTableMetadataRequest {
            share,
            schema,
            name,
            ..Default::default()
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequest<S> for QueryTableRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<serde_json::Value>(mut body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        decode_json_predicate_hints(&mut body);
        let body: QueryTableRequest = serde_json::from_value(body).map_err(|e| {
            (
                axum::http::StatusCode::BAD_REQUEST,
                format!("invalid query request: {e}"),
            )
                .into_response()
        })?;
        Ok(QueryTableRequest {
            share,
            schema,
            name,
            ..body
        })
    }
}

/// Decode a `jsonPredicateHints` sent as a JSON-encoded *string*.
///
/// The Delta Sharing protocol transmits the structured predicate as a string
/// containing JSON (that is what the reference Python / Spark clients send), while
/// the proto models it as a message. Accept both: a string that parses as JSON is
/// replaced by the parsed value; one that does not is dropped, since hints are
/// best-effort and must never fail a query.
fn decode_json_predicate_hints(body: &mut serde_json::Value) {
    let Some(obj) = body.as_object_mut() else {
        return;
    };
    for key in ["jsonPredicateHints", "json_predicate_hints"] {
        if let Some(serde_json::Value::String(raw)) = obj.get(key) {
            match serde_json::from_str::<serde_json::Value>(raw) {
                Ok(parsed) => {
                    obj.insert(key.to_string(), parsed);
                }
                Err(_) => {
                    obj.remove(key);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::decode_json_predicate_hints;
    use crate::models::open_sharing::v1::QueryTableRequest;

    #[test]
    fn json_predicate_hints_accepts_string_and_object() {
        let predicate = serde_json::json!({
            "op": "equal",
            "children": [
                {"op": "column", "name": "letter", "valueType": "string"},
                {"op": "literal", "value": "a", "valueType": "string"}
            ]
        });
        for mut body in [
            serde_json::json!({"jsonPredicateHints": predicate.to_string()}),
            serde_json::json!({"jsonPredicateHints": predicate}),
        ] {
            decode_json_predicate_hints(&mut body);
            let request: QueryTableRequest = serde_json::from_value(body).unwrap();
            let hint = request.json_predicate_hints.as_option().unwrap();
            assert_eq!(hint.op, "equal");
            assert_eq!(hint.children.len(), 2);
        }

        let mut garbage = serde_json::json!({"jsonPredicateHints": "{not json", "limitHint": 3});
        decode_json_predicate_hints(&mut garbage);
        let request: QueryTableRequest = serde_json::from_value(garbage).unwrap();
        assert!(request.json_predicate_hints.as_option().is_none());
        assert_eq!(request.limit_hint, Some(3));
    }
}
