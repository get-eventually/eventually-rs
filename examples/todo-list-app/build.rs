use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("todolist.v1_descriptor.bin"))
        .build_server(true)
        .build_client(false)
        .compile(
            &[
                "proto/todolist/v1/todo_list.proto",
                "proto/todolist/v1/todo_list_api.proto",
            ],
            &["proto"],
        )
        .unwrap();
}
