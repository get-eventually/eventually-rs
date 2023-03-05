pub mod command;
pub mod domain;
pub mod grpc;
pub mod tracing;
pub mod query;

#[allow(unused_qualifications)]
#[allow(clippy::all)] // Cannot really check the sanity of generated code :shrugs:
pub mod proto {
    tonic::include_proto!("todolist.v1");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("todolist.v1_descriptor");
}
