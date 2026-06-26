// @generated
/// Generated server implementations.
pub mod agent_skills_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with AgentSkillsServiceServer.
    #[async_trait]
    pub trait AgentSkillsService: std::marker::Send + std::marker::Sync + 'static {
        /** Lists agent skills.
*/
        async fn list_agent_skills(
            &self,
            request: tonic::Request<super::ListAgentSkillsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListAgentSkillsResponse>,
            tonic::Status,
        >;
        ///
        async fn create_agent_skill(
            &self,
            request: tonic::Request<super::CreateAgentSkillRequest>,
        ) -> std::result::Result<tonic::Response<super::AgentSkill>, tonic::Status>;
        ///
        async fn get_agent_skill(
            &self,
            request: tonic::Request<super::GetAgentSkillRequest>,
        ) -> std::result::Result<tonic::Response<super::AgentSkill>, tonic::Status>;
        ///
        async fn update_agent_skill(
            &self,
            request: tonic::Request<super::UpdateAgentSkillRequest>,
        ) -> std::result::Result<tonic::Response<super::AgentSkill>, tonic::Status>;
        ///
        async fn delete_agent_skill(
            &self,
            request: tonic::Request<super::DeleteAgentSkillRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
    }
    /** Service for managing agent skills in Unity Catalog.

 Agent skills are storage-backed directories (SKILL.md + optional resources)
 within a schema. This is an early-stage (v0alpha1) surface aligned with the
 Open Sharing agent-skill community proposal.
*/
    #[derive(Debug)]
    pub struct AgentSkillsServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> AgentSkillsServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for AgentSkillsServiceServer<T>
    where
        T: AgentSkillsService,
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
                "/unitycatalog.agent_skills.v0alpha1.AgentSkillsService/ListAgentSkills" => {
                    #[allow(non_camel_case_types)]
                    struct ListAgentSkillsSvc<T: AgentSkillsService>(pub Arc<T>);
                    impl<
                        T: AgentSkillsService,
                    > tonic::server::UnaryService<super::ListAgentSkillsRequest>
                    for ListAgentSkillsSvc<T> {
                        type Response = super::ListAgentSkillsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListAgentSkillsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AgentSkillsService>::list_agent_skills(
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
                        let method = ListAgentSkillsSvc(inner);
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
                "/unitycatalog.agent_skills.v0alpha1.AgentSkillsService/CreateAgentSkill" => {
                    #[allow(non_camel_case_types)]
                    struct CreateAgentSkillSvc<T: AgentSkillsService>(pub Arc<T>);
                    impl<
                        T: AgentSkillsService,
                    > tonic::server::UnaryService<super::CreateAgentSkillRequest>
                    for CreateAgentSkillSvc<T> {
                        type Response = super::AgentSkill;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateAgentSkillRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AgentSkillsService>::create_agent_skill(
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
                        let method = CreateAgentSkillSvc(inner);
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
                "/unitycatalog.agent_skills.v0alpha1.AgentSkillsService/GetAgentSkill" => {
                    #[allow(non_camel_case_types)]
                    struct GetAgentSkillSvc<T: AgentSkillsService>(pub Arc<T>);
                    impl<
                        T: AgentSkillsService,
                    > tonic::server::UnaryService<super::GetAgentSkillRequest>
                    for GetAgentSkillSvc<T> {
                        type Response = super::AgentSkill;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetAgentSkillRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AgentSkillsService>::get_agent_skill(&inner, request)
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
                        let method = GetAgentSkillSvc(inner);
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
                "/unitycatalog.agent_skills.v0alpha1.AgentSkillsService/UpdateAgentSkill" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateAgentSkillSvc<T: AgentSkillsService>(pub Arc<T>);
                    impl<
                        T: AgentSkillsService,
                    > tonic::server::UnaryService<super::UpdateAgentSkillRequest>
                    for UpdateAgentSkillSvc<T> {
                        type Response = super::AgentSkill;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateAgentSkillRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AgentSkillsService>::update_agent_skill(
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
                        let method = UpdateAgentSkillSvc(inner);
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
                "/unitycatalog.agent_skills.v0alpha1.AgentSkillsService/DeleteAgentSkill" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteAgentSkillSvc<T: AgentSkillsService>(pub Arc<T>);
                    impl<
                        T: AgentSkillsService,
                    > tonic::server::UnaryService<super::DeleteAgentSkillRequest>
                    for DeleteAgentSkillSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteAgentSkillRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as AgentSkillsService>::delete_agent_skill(
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
                        let method = DeleteAgentSkillSvc(inner);
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
    impl<T> Clone for AgentSkillsServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "unitycatalog.agent_skills.v0alpha1.AgentSkillsService";
    impl<T> tonic::server::NamedService for AgentSkillsServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
