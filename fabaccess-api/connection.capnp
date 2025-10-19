@0xbf017710be5a54ff;

using Rust = import "programming_language/rust.capnp";
$Rust.parentModule("schema");

using CSharp = import "programming_language/csharp.capnp";
$CSharp.namespace("FabAccessAPI.Schema");

using Authentication = import "authenticationsystem.capnp".Authentication;
using MachineSystem = import "machinesystem.capnp".MachineSystem;
using UserSystem = import "usersystem.capnp".UserSystem;
using PermissionSystem = import "permissionsystem.capnp".PermissionSystem;

const apiVersionMajor :Int32  = 0;
const apiVersionMinor :Int32  = 3;
const apiVersionPatch :Int32  = 0;

struct Version
{   
    major @0 :Int32;
    minor @1 :Int32;
    patch @2 :Int32;
}

interface Bootstrap 
{
    getAPIVersion @0 () -> Version;

    getServerRelease @1 () -> ( name :Text, release :Text );
    # Returns the server implementation name and version/build number
    # Designed only for human-facing debugging output so should be informative over machine-readable
    # Example: ( name = "bffhd", release = "0.3.1-f397e1e [rustc 1.57.0 (f1edd0429 2021-11-29)]")

    mechanisms @2 () -> ( mechs: List(Text) );
    # Get a list of Mechanisms this server allows in this context.

    createSession @3 ( mechanism :Text ) -> ( authentication :Authentication );
    # Create a new session with the server that you wish to authenticate using `mechanism`.
    # Using pipelining makes this one-roundtrip capable without explicit initial data support.
}

struct Session {
    machineSystem @0 : MachineSystem;
    userSystem @1 : UserSystem;
    permissionSystem @2 : PermissionSystem;

    vendor @3 :AnyPointer;
    # Vendor-specific APIs outside the normal API stability
}
