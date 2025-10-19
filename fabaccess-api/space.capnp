@0xbacaff4190ac7d80;

using Rust = import "programming_language/rust.capnp";
$Rust.parentModule("schema");

using CSharp = import "programming_language/csharp.capnp";
$CSharp.namespace("FabAccessAPI.Schema");

using General = import "general.capnp";

struct Space 
{
    id @0 :General.UUID;
    name @1 :Text;
    info @2 :Text;
}