@0xd0568a21cf11488e;

using Rust = import "programming_language/rust.capnp";
$Rust.parentModule("schema");

using CSharp = import "programming_language/csharp.capnp";
$CSharp.namespace("FabAccessAPI.Schema");

using Role = import "role.capnp".Role;

struct PermissionSystem
{
    info @0 :Info;
    interface Info $CSharp.name("InfoInterface") {
    	getRoleList @0 () -> ( role_list :List(Role) );
	}

	manage @1 :Manage;
    interface Manage $CSharp.name("ManageInterface") {
    	
	}
}