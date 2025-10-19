@0x9a05e95f65f2edda;

using Rust = import "programming_language/rust.capnp";
$Rust.parentModule("schema");

using CSharp = import "programming_language/csharp.capnp";
$CSharp.namespace("FabAccessAPI.Schema");

using General = import "general.capnp";
using User = import "user.capnp".User;
using Optional = General.Optional;
using Fallible = General.Fallible;

struct UserSystem
{
	info @0 :Info;
    interface Info $CSharp.name("InfoInterface") {
    	getUserSelf @0 ( ) -> User;
	}

	search @2 :Search;
	interface Search $CSharp.name("SearchInterface") {
	    getUserByName @0 (username: Text) -> Optional(User);
	}

	manage @1 :Manage;
    interface Manage $CSharp.name("ManageInterface") {
    	getUserList @0 () -> ( user_list :List(User) );

        addUser @1 (username :Text, password: Text) -> User;
        # DEPRECATED: use `addUserFallible` instead

    	removeUser @2 (user: User);

        struct AddUserError {
            enum AddUserError $CSharp.name("AddUserErrorEnum") {
                alreadyExists @0;
                # An user with that username already exists

                usernameInvalid @1;
                # The provided username is unusable, e.g. contains invalid characters,
                # is too long or too short.

                passwordInvalid @2;
                # The provided password is unusable, e.g. it's of zero length
            }
            error @0 :AddUserError;
        }
    	addUserFallible @3 (username :Text, password: Text) -> Fallible(User, AddUserError);
	}
}
