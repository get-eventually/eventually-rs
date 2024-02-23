pub mod application;
pub mod domain;
pub mod grpc;
pub mod postgres;
pub mod serde;
pub mod tracing;

#[allow(unused_qualifications)]
#[allow(clippy::all)] // Cannot really check the sanity of generated code :shrugs:
pub mod proto {
    tonic::include_proto!("bankaccounting");
    tonic::include_proto!("bankaccount");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("bankaccouting_descriptor");
}
