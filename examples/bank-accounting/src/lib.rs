#![deny(unsafe_code, unused_qualifications, trivial_casts)]
#![deny(clippy::all)]
// #![warn(clippy::pedantic)]

pub mod application;
pub mod domain;
pub mod grpc;

#[allow(unused_qualifications)]
pub mod proto {
    tonic::include_proto!("bankaccounting");

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("bankaccouting_descriptor");
}
