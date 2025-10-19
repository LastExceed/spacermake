fn main() {
    capnpc::CompilerCommand::new()
        .import_path("fabaccess-api/")
        .src_prefix("fabaccess-api/")
        .file("fabaccess-api/authenticationsystem.capnp")
        .file("fabaccess-api/connection.capnp")
        .file("fabaccess-api/general.capnp")
        .file("fabaccess-api/machine.capnp")
        .file("fabaccess-api/machinesystem.capnp")
        .file("fabaccess-api/permissionsystem.capnp")
        .file("fabaccess-api/role.capnp")
        .file("fabaccess-api/space.capnp")
        .file("fabaccess-api/user.capnp")
        .file("fabaccess-api/usersystem.capnp")
        .run()
        .unwrap();
}