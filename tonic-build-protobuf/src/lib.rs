#![doc = include_str!("../README.md")]

use core::fmt;
use std::{
    fs,
    path::{Path, PathBuf},
};

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream;
use protobuf::descriptor;
use quote::ToTokens;
use tonic_build::CodeGenBuilder;

/// A service descriptor.
#[derive(Debug, Default)]
struct Service {
    /// The service name in Rust style.
    name: String,
    /// The package name as it appears in the .proto file.
    package: String,
    /// The service methods.
    methods: Vec<Method>,
}

impl tonic_build::Service for Service {
    type Comment = String;

    type Method = Method;

    fn name(&self) -> &str {
        &self.name
    }

    fn package(&self) -> &str {
        &self.package
    }

    fn identifier(&self) -> &str {
        &self.name
    }

    fn methods(&self) -> &[Self::Method] {
        &self.methods
    }

    fn comment(&self) -> &[Self::Comment] {
        &[]
    }
}

/// A service method descriptor.
#[derive(Debug, Default)]
struct Method {
    /// The name of the method in Rust style.
    name: String,
    /// The name of the method as should be used when constructing a route
    route_name: String,
    /// The input Rust type.
    input_type: String,
    /// The output Rust type.
    output_type: String,
    /// Identifies if client streams multiple client messages.
    client_streaming: bool,
    /// Identifies if server streams multiple server messages.
    server_streaming: bool,
    /// The path to the codec to use for this method
    codec_path: String,
}

impl tonic_build::Method for Method {
    type Comment = String;

    fn name(&self) -> &str {
        &self.name
    }

    fn identifier(&self) -> &str {
        &self.route_name
    }

    fn codec_path(&self) -> &str {
        &self.codec_path
    }

    fn client_streaming(&self) -> bool {
        self.client_streaming
    }

    fn server_streaming(&self) -> bool {
        self.server_streaming
    }

    fn comment(&self) -> &[Self::Comment] {
        &[]
    }

    fn request_response_name(
        &self,
        proto_path: &str,
        _compile_well_known_types: bool,
    ) -> (TokenStream, TokenStream) {
        let convert_type = |rust_type: &str| -> TokenStream {
            if rust_type.is_empty() {
                syn::parse_str::<syn::Path>(rust_type)
                    .unwrap()
                    .to_token_stream()
            } else {
                syn::parse_str::<syn::Path>(&format!("{}{}", proto_path, rust_type))
                    .unwrap()
                    .to_token_stream()
            }
        };

        let request = convert_type(&self.input_type);
        let response = convert_type(&self.output_type);
        (request, response)
    }
}

struct ServiceGenerator {
    builder: Builder,
    clients: TokenStream,
    servers: TokenStream,
}

impl ServiceGenerator {
    fn generate(&mut self, service: &Service) {
        if self.builder.build_server {
            let server = CodeGenBuilder::new()
                .emit_package(true)
                .compile_well_known_types(false)
                .generate_server(service, &self.builder.proto_path);

            self.servers.extend(server);
        }

        if self.builder.build_client {
            let client = CodeGenBuilder::new()
                .emit_package(true)
                .compile_well_known_types(false)
                .build_transport(self.builder.build_transport)
                .generate_client(service, &self.builder.proto_path);

            self.clients.extend(client);
        }
    }

    fn finalize(&mut self, buf: &mut String) {
        if self.builder.build_client && !self.clients.is_empty() {
            let clients = &self.clients;

            let client_service = quote::quote! {
                #clients
            };

            let ast: syn::File = syn::parse2(client_service).expect("not a valid tokenstream");
            let code = prettyplease::unparse(&ast);
            buf.push_str(&code);

            self.clients = TokenStream::default();
        }

        if self.builder.build_server && !self.servers.is_empty() {
            let servers = &self.servers;

            let server_service = quote::quote! {
                #servers
            };

            let ast: syn::File = syn::parse2(server_service).expect("not a valid tokenstream");
            let code = prettyplease::unparse(&ast);
            buf.push_str(&code);

            self.servers = TokenStream::default();
        }
    }
}

#[allow(clippy::type_complexity)]
struct FileNameFn(Box<dyn Fn(&str, &str) -> String>);

impl fmt::Debug for FileNameFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileNameFn(...)")
    }
}

/// Service generator builder.
#[derive(Debug)]
pub struct Builder {
    proto_path: String,
    file_name_fn: Option<FileNameFn>,
    build_server: bool,
    build_client: bool,
    build_transport: bool,
    codec_path: String,

    out_dir: Option<PathBuf>,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            proto_path: "super".to_owned(),
            codec_path: "::tonic_codec_protobuf::ProtobufCodecV3".to_string(),
            file_name_fn: Some(FileNameFn(Box::new(|package_name, service_name| {
                format!("{}_{}", package_name, service_name)
            }))),
            build_server: true,
            build_client: true,
            build_transport: true,
            out_dir: None,
        }
    }
}

impl Builder {
    /// Create a new Builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the default codec.
    ///
    /// If set, writes `{codec_path}::default()` in generated code wherever a
    /// codec is created.
    ///
    /// This defaults to `"::tonic_codec_protobuf::ProtobufCodecV3"`
    pub fn codec_path(mut self, codec_path: impl AsRef<str>) -> Self {
        self.codec_path = codec_path.as_ref().to_string();
        self
    }

    /// Set the path to where the generated code will search for the
    /// Request/Response proto structs live relative to the module where you
    /// call `include_proto!`.
    ///
    /// This defaults to `super`.
    pub fn proto_path(mut self, proto_path: impl AsRef<str>) -> Self {
        self.proto_path = proto_path.as_ref().to_string();
        self
    }

    /// Specify names of generated rust files. The `file_name_fn` is provided
    /// with `package_name` and `service_name`, and it should return a name
    /// without ".rs" extension.
    ///
    /// This defaults to `"{package_name}_{service_name}"`.
    pub fn file_name<F>(mut self, file_name_fn: F) -> Self
    where
        F: Fn(&str, &str) -> String + 'static,
    {
        self.file_name_fn = Some(FileNameFn(Box::new(file_name_fn)));
        self
    }

    /// Enable or disable gRPC client code generation.
    ///
    /// Defaults to enabling client code generation.
    pub fn build_client(mut self, enable: bool) -> Self {
        self.build_client = enable;
        self
    }

    /// Enable or disable gRPC server code generation.
    ///
    /// Defaults to enabling server code generation.
    pub fn build_server(mut self, enable: bool) -> Self {
        self.build_server = enable;
        self
    }

    /// Enable or disable generated clients and servers to have built-in tonic
    /// transport features.
    ///
    /// When the `transport` feature is disabled this does nothing.
    pub fn build_transport(mut self, enable: bool) -> Self {
        self.build_transport = enable;
        self
    }

    /// Set the output directory to generate code to.
    ///
    /// Defaults to the `OUT_DIR` environment variable.
    pub fn out_dir(mut self, out_dir: impl AsRef<Path>) -> Self {
        self.out_dir = Some(out_dir.as_ref().to_path_buf());
        self
    }

    /// Performs code generation for the provided services.
    ///
    /// Generated services will be output into the directory specified by
    /// `out_dir` with files named specified by [`Builder::file_name`].
    pub fn compile(self, protos: &[impl AsRef<Path>], includes: &[impl AsRef<Path>]) {
        let fds = self.build_file_descriptor_set(protos, includes);
        let mut services = vec![];
        for fd in fds.file {
            services.extend(self.build_services(fd));
        }
        self.compile_svc(&services);
    }

    fn build_file_descriptor_set(
        &self,
        protos: &[impl AsRef<Path>],
        includes: &[impl AsRef<Path>],
    ) -> descriptor::FileDescriptorSet {
        protobuf_parse::Parser::new()
            .protoc()
            .inputs(protos)
            .includes(includes)
            .file_descriptor_set()
            .expect("protoc failed")
    }

    /// Performs code generation for the provided services.
    fn compile_svc(mut self, services: &[Service]) {
        let out_dir = if let Some(out_dir) = self.out_dir.as_ref() {
            out_dir.clone()
        } else {
            PathBuf::from(std::env::var("OUT_DIR").unwrap())
        };

        let file_name = self.file_name_fn.take().unwrap();
        let mut generator = ServiceGenerator {
            builder: self,
            clients: TokenStream::default(),
            servers: TokenStream::default(),
        };

        for service in services {
            generator.generate(service);
            let mut output = String::new();
            generator.finalize(&mut output);

            let file_name = (file_name.0)(&service.package, &service.name);
            let mod_name = rust_mod_name_convention(&file_name);
            let out_file = out_dir.join(&format!("{}.rs", mod_name));
            fs::write(out_file, output).unwrap();
        }
    }

    /// Build services from the provided `FileDescriptorProto`.
    fn build_services(&self, fd: descriptor::FileDescriptorProto) -> Vec<Service> {
        let package_name = &protobuf_path_to_rust_mod(fd.package());

        let mut services = vec![];
        for svc in &fd.service {
            let build_method = |m: &descriptor::MethodDescriptorProto| Method {
                name: rust_method_name_convention(m.name()),
                route_name: m.name().to_owned(),
                input_type: protobuf_path_to_rust_path(m.input_type()),
                output_type: protobuf_path_to_rust_path(m.output_type()),
                codec_path: self.codec_path.to_owned(),
                client_streaming: m.client_streaming(),
                server_streaming: m.server_streaming(),
            };
            let build_service = |svc: &descriptor::ServiceDescriptorProto| Service {
                name: svc.name().to_owned(),
                package: package_name.to_owned(),
                methods: svc.method.iter().map(build_method).collect(),
            };
            services.push(build_service(svc));
        }

        services
    }
}

fn rust_mod_name_convention(name: &str) -> String {
    name.to_snake_case()
}

fn rust_method_name_convention(name: &str) -> String {
    name.to_snake_case()
}

fn rust_struct_name_convention(name: &str) -> String {
    name.to_upper_camel_case()
}

// ".package_1.package_2.package_3" -> "package_3"
fn protobuf_path_to_rust_mod(path: &str) -> String {
    path.split('.').last().unwrap().to_owned()
}

// ".package.Message" -> "::package::Message"
fn protobuf_path_to_rust_path(path: &str) -> String {
    let mut rust_path = String::new();
    let mut parts = path.split('.');
    let mut last_item = parts.next();
    loop {
        let Some(item) = parts.next() else {
            break;
        };
        if last_item.unwrap().is_empty() {
            // Skip root.
            last_item = Some(item);
            continue;
        }
        rust_path.push_str("::");
        rust_path.push_str(last_item.unwrap());
        last_item = Some(item);
    }
    rust_path.push_str("::");
    rust_path.push_str(&rust_struct_name_convention(last_item.unwrap()));
    rust_path
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_streaming_rpc() {
        let proto_content = r#"
            syntax = "proto3";
            package testing;
            service Streaming {
                rpc GetUnary(GetRequest) returns (GetResponse) {}
                rpc GetClientStreaming(stream GetRequest) returns (GetResponse) {}
                rpc GetServerStreaming(GetRequest) returns (stream GetResponse) {}
                rpc GetBidirectionalStreaming(stream GetRequest) returns (stream GetResponse) {}
            }
            message GetRequest {}
            message GetResponse {}
        "#;

        let tmp_dir = tempfile::TempDir::new().unwrap();
        let proto_file_path = tmp_dir.path().join("test_streaming_rpc.proto");
        std::fs::write(&proto_file_path, proto_content).unwrap();

        let fds = crate::Builder::new()
            .out_dir(tmp_dir.path())
            .build_file_descriptor_set(&[proto_file_path], &[tmp_dir.path()]);
        assert_eq!(fds.file[0].service.len(), 1);
        assert_eq!(fds.file[0].service[0].method.len(), 4);

        let assert = |rpc: &str, client_streaming, server_streaming| {
            let method = fds.file[0].service[0]
                .method
                .iter()
                .find(|m| m.name() == rpc)
                .unwrap();
            assert_eq!(method.client_streaming(), client_streaming, "{fds}");
            assert_eq!(method.server_streaming(), server_streaming, "{fds}");
        };

        // Unary
        assert("GetUnary", false, false);
        // Client streaming
        assert("GetClientStreaming", true, false);
        // Server streaming
        assert("GetServerStreaming", false, true);
        // Bidirectional Streaming
        assert("GetBidirectionalStreaming", true, true);
    }
}
