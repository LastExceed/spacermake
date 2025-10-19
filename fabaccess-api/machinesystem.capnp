@0xe89d197dcef9c49b;

using Rust = import "programming_language/rust.capnp";
$Rust.parentModule("schema");

using CSharp = import "programming_language/csharp.capnp";
$CSharp.namespace("FabAccessAPI.Schema");

using General = import "general.capnp";
using Optional = General.Optional;
using Machine = import "machine.capnp".Machine;

struct MachineSystem
{
	info @0 :Info;
    interface Info $CSharp.name("InfoInterface") {
    	getMachineList @0 () -> ( machine_list :List(Machine) );

    	getMachine @1 ( id :Text ) -> Optional(Machine);
		getMachineURN @2 ( urn :Text ) -> Optional(Machine);
	}
}
