// @generated
/// Generated server implementations.
pub mod tag_policies_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with TagPoliciesServiceServer.
    #[async_trait]
    pub trait TagPoliciesService: std::marker::Send + std::marker::Sync + 'static {
        async fn list_tag_policies(
            &self,
            request: tonic::Request<super::ListTagPoliciesRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListTagPoliciesResponse>,
            tonic::Status,
        >;
        async fn create_tag_policy(
            &self,
            request: tonic::Request<super::CreateTagPolicyRequest>,
        ) -> std::result::Result<tonic::Response<super::TagPolicy>, tonic::Status>;
        async fn get_tag_policy(
            &self,
            request: tonic::Request<super::GetTagPolicyRequest>,
        ) -> std::result::Result<tonic::Response<super::TagPolicy>, tonic::Status>;
        async fn update_tag_policy(
            &self,
            request: tonic::Request<super::UpdateTagPolicyRequest>,
        ) -> std::result::Result<tonic::Response<super::TagPolicy>, tonic::Status>;
        async fn delete_tag_policy(
            &self,
            request: tonic::Request<super::DeleteTagPolicyRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct TagPoliciesServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> TagPoliciesServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for TagPoliciesServiceServer<T>
    where
        T: TagPoliciesService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/unitycatalog.tags.v1.TagPoliciesService/ListTagPolicies" => {
                    #[allow(non_camel_case_types)]
                    struct ListTagPoliciesSvc<T: TagPoliciesService>(pub Arc<T>);
                    impl<
                        T: TagPoliciesService,
                    > tonic::server::UnaryService<super::ListTagPoliciesRequest>
                    for ListTagPoliciesSvc<T> {
                        type Response = super::ListTagPoliciesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListTagPoliciesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TagPoliciesService>::list_tag_policies(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListTagPoliciesSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.tags.v1.TagPoliciesService/CreateTagPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct CreateTagPolicySvc<T: TagPoliciesService>(pub Arc<T>);
                    impl<
                        T: TagPoliciesService,
                    > tonic::server::UnaryService<super::CreateTagPolicyRequest>
                    for CreateTagPolicySvc<T> {
                        type Response = super::TagPolicy;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateTagPolicyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TagPoliciesService>::create_tag_policy(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateTagPolicySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.tags.v1.TagPoliciesService/GetTagPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct GetTagPolicySvc<T: TagPoliciesService>(pub Arc<T>);
                    impl<
                        T: TagPoliciesService,
                    > tonic::server::UnaryService<super::GetTagPolicyRequest>
                    for GetTagPolicySvc<T> {
                        type Response = super::TagPolicy;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetTagPolicyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TagPoliciesService>::get_tag_policy(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetTagPolicySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.tags.v1.TagPoliciesService/UpdateTagPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateTagPolicySvc<T: TagPoliciesService>(pub Arc<T>);
                    impl<
                        T: TagPoliciesService,
                    > tonic::server::UnaryService<super::UpdateTagPolicyRequest>
                    for UpdateTagPolicySvc<T> {
                        type Response = super::TagPolicy;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateTagPolicyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TagPoliciesService>::update_tag_policy(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = UpdateTagPolicySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.tags.v1.TagPoliciesService/DeleteTagPolicy" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteTagPolicySvc<T: TagPoliciesService>(pub Arc<T>);
                    impl<
                        T: TagPoliciesService,
                    > tonic::server::UnaryService<super::DeleteTagPolicyRequest>
                    for DeleteTagPolicySvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteTagPolicyRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TagPoliciesService>::delete_tag_policy(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeleteTagPolicySvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for TagPoliciesServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "unitycatalog.tags.v1.TagPoliciesService";
    impl<T> tonic::server::NamedService for TagPoliciesServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
/// Generated server implementations.
pub mod entity_tag_assignments_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with EntityTagAssignmentsServiceServer.
    #[async_trait]
    pub trait EntityTagAssignmentsService: std::marker::Send + std::marker::Sync + 'static {
        /** List entity tag assignments

 Gets the tag assignments for the specified entity.
*/
        async fn list_entity_tag_assignments(
            &self,
            request: tonic::Request<super::ListEntityTagAssignmentsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListEntityTagAssignmentsResponse>,
            tonic::Status,
        >;
        /** Create an entity tag assignment

 Assigns a tag to a Unity Catalog entity.
*/
        async fn create_entity_tag_assignment(
            &self,
            request: tonic::Request<super::CreateEntityTagAssignmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EntityTagAssignment>,
            tonic::Status,
        >;
        /** Get an entity tag assignment

 Gets the tag assignment for the specified entity and tag key.
*/
        async fn get_entity_tag_assignment(
            &self,
            request: tonic::Request<super::GetEntityTagAssignmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EntityTagAssignment>,
            tonic::Status,
        >;
        /** Update an entity tag assignment

 Updates the tag assignment for the specified entity and tag key.
*/
        async fn update_entity_tag_assignment(
            &self,
            request: tonic::Request<super::UpdateEntityTagAssignmentRequest>,
        ) -> std::result::Result<
            tonic::Response<super::EntityTagAssignment>,
            tonic::Status,
        >;
        /** Delete an entity tag assignment

 Deletes the tag assignment for the specified entity and tag key.
*/
        async fn delete_entity_tag_assignment(
            &self,
            request: tonic::Request<super::DeleteEntityTagAssignmentRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
    }
    /** Manage assignments of tags to Unity Catalog entities.
*/
    #[derive(Debug)]
    pub struct EntityTagAssignmentsServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> EntityTagAssignmentsServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>>
    for EntityTagAssignmentsServiceServer<T>
    where
        T: EntityTagAssignmentsService,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/unitycatalog.tags.v1.EntityTagAssignmentsService/ListEntityTagAssignments" => {
                    #[allow(non_camel_case_types)]
                    struct ListEntityTagAssignmentsSvc<T: EntityTagAssignmentsService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: EntityTagAssignmentsService,
                    > tonic::server::UnaryService<super::ListEntityTagAssignmentsRequest>
                    for ListEntityTagAssignmentsSvc<T> {
                        type Response = super::ListEntityTagAssignmentsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::ListEntityTagAssignmentsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EntityTagAssignmentsService>::list_entity_tag_assignments(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListEntityTagAssignmentsSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.tags.v1.EntityTagAssignmentsService/CreateEntityTagAssignment" => {
                    #[allow(non_camel_case_types)]
                    struct CreateEntityTagAssignmentSvc<T: EntityTagAssignmentsService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: EntityTagAssignmentsService,
                    > tonic::server::UnaryService<
                        super::CreateEntityTagAssignmentRequest,
                    > for CreateEntityTagAssignmentSvc<T> {
                        type Response = super::EntityTagAssignment;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::CreateEntityTagAssignmentRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EntityTagAssignmentsService>::create_entity_tag_assignment(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateEntityTagAssignmentSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.tags.v1.EntityTagAssignmentsService/GetEntityTagAssignment" => {
                    #[allow(non_camel_case_types)]
                    struct GetEntityTagAssignmentSvc<T: EntityTagAssignmentsService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: EntityTagAssignmentsService,
                    > tonic::server::UnaryService<super::GetEntityTagAssignmentRequest>
                    for GetEntityTagAssignmentSvc<T> {
                        type Response = super::EntityTagAssignment;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetEntityTagAssignmentRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EntityTagAssignmentsService>::get_entity_tag_assignment(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetEntityTagAssignmentSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.tags.v1.EntityTagAssignmentsService/UpdateEntityTagAssignment" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateEntityTagAssignmentSvc<T: EntityTagAssignmentsService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: EntityTagAssignmentsService,
                    > tonic::server::UnaryService<
                        super::UpdateEntityTagAssignmentRequest,
                    > for UpdateEntityTagAssignmentSvc<T> {
                        type Response = super::EntityTagAssignment;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::UpdateEntityTagAssignmentRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EntityTagAssignmentsService>::update_entity_tag_assignment(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = UpdateEntityTagAssignmentSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/unitycatalog.tags.v1.EntityTagAssignmentsService/DeleteEntityTagAssignment" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteEntityTagAssignmentSvc<T: EntityTagAssignmentsService>(
                        pub Arc<T>,
                    );
                    impl<
                        T: EntityTagAssignmentsService,
                    > tonic::server::UnaryService<
                        super::DeleteEntityTagAssignmentRequest,
                    > for DeleteEntityTagAssignmentSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::DeleteEntityTagAssignmentRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as EntityTagAssignmentsService>::delete_entity_tag_assignment(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeleteEntityTagAssignmentSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for EntityTagAssignmentsServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "unitycatalog.tags.v1.EntityTagAssignmentsService";
    impl<T> tonic::server::NamedService for EntityTagAssignmentsServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
