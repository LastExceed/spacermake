@0xff5b4a767d98592a;

using Rust = import "programming_language/rust.capnp";
$Rust.parentModule("schema");

using CSharp = import "programming_language/csharp.capnp";
$CSharp.namespace("FabAccessAPI.Schema");

struct UUID {
    # UUID type used to identify machines.
    # Since the exact value has no meaning the encoding rules are not too relevant, but it is
    # paramount that you are consistent when encoding and decoding this type.
    #
    # Consider using this algorithm for assembling the 128-bit integer:
    # (assuming ISO9899:2018 shifting & casting rules)
    #   uint128_t num = (uuid1 << 64) + uuid0;
    # And then respectively this code for deconstructing it:
    #   uint64_t uuid0 = (uint64_t) num;
    #   uint64_t uuid1 = (uint64_t) (num >> 64);

    uuid0 @0 :UInt64;
    uuid1 @1 :UInt64;
}

struct KeyValuePair {
    key @0 :Text;
    value @1 :Text;
}

struct Optional(T) {
    union {
        nothing @0 :Void;
        just @1 :T;
    }
}

struct Fallible(T, E) {
    # Some operations can fail in several expected ways.
    # In those cases returning an `Optional` doesn't transfer information about the way that the
    # operation failed. `Fallible` contains this information in the generic `E` type.
    union {
        failed @0 :E;
        successful @1 :T;
    }
}