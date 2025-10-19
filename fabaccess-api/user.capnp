@0xc7941adf5db6bbf0;

using Rust = import "programming_language/rust.capnp";
$Rust.parentModule("schema");

using CSharp = import "programming_language/csharp.capnp";
$CSharp.namespace("FabAccessAPI.Schema");

using General = import "general.capnp";
using Space = import "space.capnp".Space;
using Role = import "role.capnp".Role;

struct User
{
    id @0 :General.UUID;
    username @1 :Text;
    space @2 :Space;

    struct UserInfoExtended
    {
        id @0 :General.UUID;
        name @1 :Text;
    }

    info @3 :Info;
    interface Info $CSharp.name("InfoInterface") {
        listRoles @0 () -> ( roles :List(Role) );
    }

    manage @4 :Manage;
    interface Manage $CSharp.name("ManageInterface") {
        pwd @0 ( old_pwd :Text, new_pwd :Text ) -> ();
    }

    admin @5 :Admin;
    interface Admin $CSharp.name("AdminInterface") {
        getUserInfoExtended @0 () -> ( userInfoExtended :UserInfoExtended );
        
        addRole @1 ( role :Role ) -> ();
        removeRole @2 ( role :Role ) -> ();

        pwd @3 ( new_pwd :Text ) -> ();
    }

    cardDESFireEV2 @6 :CardDESFireEV2;
    interface CardDESFireEV2 $CSharp.name("CardDESFireInterface") {
        # For more details about FabFire specification please see:
        # https://docs.fab-access.org/books/fabfire-und-nfc-tags/page/fabfire-funktionsprinzip-grundlagen 

        getTokenList @0 () -> ( token_list :List(Data) );
        # Get a list of all user Token currently bound to an user. This will generally be the number
        # of cards they use.

        bind @1 ( token :Data, auth_key :Data ) -> ();
        # Bind a given URL to a given auth key. The server will store both URL and key, so using
        # this frequently will force the server to store large amounts of data. 
        # Trying to bind a new key to an existing URL will fail.

        unbind @2 ( token :Data ) -> ();
        # Unbind the key associated with the given token. This will fail all future attempts to use
        # the card with the associated key.

        genCardToken @3 () -> ( token :Data );
        # Generate a new Token that can be used to access an user in a pseudonymized fashion.
        # This call is extremely cheap to make as the server will not store this Token.

        getMetaInfo @4 () -> ( bytes :Data );
        # Retrieve the blob for File 0001 from the server. The returned bytes are in the correct
        # format to be written to the card as-is.

        getSpaceInfo @5 () -> ( bytes :Data );
        # Retrieve the blob for File 0002 from the server. The returned bytes are in the correct
        # format to be written to the card as-is, but a client MAY add or change some information
        # contained.
    }
}