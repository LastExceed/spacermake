@0x8c2f829df1930cd5;

using Rust = import "programming_language/rust.capnp";
$Rust.parentModule("schema");

using CSharp = import "programming_language/csharp.capnp";
$CSharp.namespace("FabAccessAPI.Schema");

using General = import "general.capnp";
using Optional = General.Optional;
using User = import "user.capnp".User;
using Space = import "space.capnp".Space;

struct Machine {
    enum MachineState {
        free @0;
        inUse @1;
        toCheck @2;
        blocked @3;
        disabled @4;
        reserved @5;
        totakeover @6;
    }
    struct MachineInfoExtended {
        currentUser @0 :Optional(User);
        lastUser @1 :Optional(User);
        instructorUser @2 :Optional(User);
    }

    struct Reservation {
        user @0 :User;
        start @1: UInt64;
        end @2: UInt64;
    }

    id @0 :Text;
    space @1 :Space;
    name @2 :Text;
    description @3 :Text;
    state @4 :MachineState;
    manager @5:Optional(User);
    wiki @13 :Text;
    urn @14 :Text;
    category @15 :Text;

    info @6 :Info;
    interface Info $CSharp.name("InfoInterface") {
        getPropertyList @0 () -> ( propertyList :List(General.KeyValuePair) );

        getReservationList @1 () -> ( reservationList :List(Reservation) );
    }

    use @7 :Use;
    interface Use $CSharp.name("UseInterface") {
        use @0 ();

        reserve @1 ();
        reserveto @2 (start :UInt64, end :UInt64);
    }

    inuse @8 :InUse;
    interface InUse $CSharp.name("InUseInterface") {
        giveBack @0 ();
        sendRawData @1 (data :Data);
        releasefortakeover @2 ();
    }

    prodable @16 :Prodable;
    interface Prodable $CSharp.name("ProdInterface") {
        prodWithData @0 (data :Data);
    }

    takeover @9 :Takeover;
    interface Takeover $CSharp.name("TakeoverInterface") {
        accept @0 ();
        reject @1 ();
    }

    check @10 :Check;
    interface Check $CSharp.name("CheckInterface") {
        check @0 ();
        reject @1 ();
    }

    manage @11 :Manage;
    interface Manage $CSharp.name("ManageInterface") {
        getMachineInfoExtended @0 () -> MachineInfoExtended;
        
        setProperty @1 (property :General.KeyValuePair);
        removeProperty @2 (property :General.KeyValuePair);

        forceUse @3 ();
        forceFree @4 ();

        forceTransfer @5 (user :User);

        block @6 ();
        disabled @7 ();
    }

    admin @12 :Admin;
    interface Admin $CSharp.name("AdminInterface") {
        forceSetState @0 ( state :MachineState );
        forceSetUser @1 ( user :User );

        getAdminPropertyList @2 () -> (propertyList :List(General.KeyValuePair));
        setAdminProperty @3 (property :General.KeyValuePair);
        removeAdminProperty @4 (property :General.KeyValuePair);
    }
}
