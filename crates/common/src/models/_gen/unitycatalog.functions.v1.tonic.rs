// @generated
/// Generated server implementations.
pub mod functions_service_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with FunctionsServiceServer.
    #[async_trait]
    pub trait FunctionsService: std::marker::Send + std::marker::Sync + 'static {
        /** List functions

 List functions within the specified parent catalog and schema. If the caller is the metastore
 admin, all functions are returned in the response. Otherwise, the caller must have USE_CATALOG
 on the parent catalog and USE_SCHEMA on the parent schema, and the function must either be
 owned by the caller or have SELECT on the function.
*/
        async fn list_functions(
            &self,
            request: tonic::Request<super::ListFunctionsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListFunctionsResponse>,
            tonic::Status,
        >;
        /** Create a function

 Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION
 privilege on the parent catalog and schema.
*/
        async fn create_function(
            &self,
            request: tonic::Request<super::CreateFunctionRequest>,
        ) -> std::result::Result<tonic::Response<super::Function>, tonic::Status>;
        /** Get a function

 Gets a function from within a parent catalog and schema. For the fetch to succeed,
 the caller must be a metastore admin, the owner of the function, or have SELECT on
 the function.
*/
        async fn get_function(
            &self,
            request: tonic::Request<super::GetFunctionRequest>,
        ) -> std::result::Result<tonic::Response<super::Function>, tonic::Status>;
        /** Update a function

 Updates the function that matches the supplied name. Only the owner of the function
 can be updated.
*/
        async fn update_function(
            &self,
            request: tonic::Request<super::UpdateFunctionRequest>,
        ) -> std::result::Result<tonic::Response<super::Function>, tonic::Status>;
        /** Delete a function

 Deletes the function that matches the supplied name. For the deletion to succeed,
 the caller must be the owner of the function.
*/
        async fn delete_function(
            &self,
            request: tonic::Request<super::DeleteFunctionRequest>,
        ) -> std::result::Result<tonic::Response<()>, tonic::Status>;
    }
    /** Manage User-Defined Functions (UDFs) in the service.
*/
    #[derive(Debug)]
    pub struct FunctionsServiceServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> FunctionsServiceServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for FunctionsServiceServer<T>
    where
        T: FunctionsService,
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
                "/unitycatalog.functions.v1.FunctionsService/ListFunctions" => {
                    #[allow(non_camel_case_types)]
                    struct ListFunctionsSvc<T: FunctionsService>(pub Arc<T>);
                    impl<
                        T: FunctionsService,
                    > tonic::server::UnaryService<super::ListFunctionsRequest>
                    for ListFunctionsSvc<T> {
                        type Response = super::ListFunctionsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListFunctionsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FunctionsService>::list_functions(&inner, request)
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
                        let method = ListFunctionsSvc(inner);
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
                "/unitycatalog.functions.v1.FunctionsService/CreateFunction" => {
                    #[allow(non_camel_case_types)]
                    struct CreateFunctionSvc<T: FunctionsService>(pub Arc<T>);
                    impl<
                        T: FunctionsService,
                    > tonic::server::UnaryService<super::CreateFunctionRequest>
                    for CreateFunctionSvc<T> {
                        type Response = super::Function;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateFunctionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FunctionsService>::create_function(&inner, request)
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
                        let method = CreateFunctionSvc(inner);
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
                "/unitycatalog.functions.v1.FunctionsService/GetFunction" => {
                    #[allow(non_camel_case_types)]
                    struct GetFunctionSvc<T: FunctionsService>(pub Arc<T>);
                    impl<
                        T: FunctionsService,
                    > tonic::server::UnaryService<super::GetFunctionRequest>
                    for GetFunctionSvc<T> {
                        type Response = super::Function;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetFunctionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FunctionsService>::get_function(&inner, request).await
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
                        let method = GetFunctionSvc(inner);
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
                "/unitycatalog.functions.v1.FunctionsService/UpdateFunction" => {
                    #[allow(non_camel_case_types)]
                    struct UpdateFunctionSvc<T: FunctionsService>(pub Arc<T>);
                    impl<
                        T: FunctionsService,
                    > tonic::server::UnaryService<super::UpdateFunctionRequest>
                    for UpdateFunctionSvc<T> {
                        type Response = super::Function;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UpdateFunctionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FunctionsService>::update_function(&inner, request)
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
                        let method = UpdateFunctionSvc(inner);
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
                "/unitycatalog.functions.v1.FunctionsService/DeleteFunction" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteFunctionSvc<T: FunctionsService>(pub Arc<T>);
                    impl<
                        T: FunctionsService,
                    > tonic::server::UnaryService<super::DeleteFunctionRequest>
                    for DeleteFunctionSvc<T> {
                        type Response = ();
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteFunctionRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as FunctionsService>::delete_function(&inner, request)
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
                        let method = DeleteFunctionSvc(inner);
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
    impl<T> Clone for FunctionsServiceServer<T> {
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
    pub const SERVICE_NAME: &str = "unitycatalog.functions.v1.FunctionsService";
    impl<T> tonic::server::NamedService for FunctionsServiceServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
