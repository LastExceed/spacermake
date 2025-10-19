@0xb9cffd29ac983e9f;


using Rust = import "programming_language/rust.capnp";
$Rust.parentModule("schema");

using CSharp = import "programming_language/csharp.capnp";
$CSharp.namespace("FabAccessAPI.Schema");

using Session = import "connection.capnp".Session;

struct Response {
    enum Error {
        aborted @0;
        # This authentication exchange was aborted by either side.

        badMechanism @1;
        # The server does not support this mechanism in this context.

        invalidCredentials @2;
        # The exchange was valid, but the provided credentials are invalid. This may mean that the
        # authcid is not known to the server or that the password/certificate/key/ticket/etc is not
        # correct.

        failed @3;
        # A generic failed result. A server sending this result MUST set the `action` field to
        # indicate whether this is a temporary or permanent failure and SHOULD set `description` to
        # a human-readable error message.
    }
    union {
        failed :group {
            code @0 :Error;
            # Error code indicating the cause of the error

            additionalData @1 :Data;
            # Some mechanisms will send additional data after an error
        }
        # Some kind of error happened. This in the first entry in the union because it is the
        # default set value meaning if a server fails to set any of the values, indicating some
        # pretty severe server bugs, it is parsed as an "aborted" error.
        
        challenge @2 :Data;
        # The data provided so far is not enough to authenticate the user. The data MAY be a
        # NULL-ptr or a non-NULL list ptr of zero bytes which clients MUST pass to their SASL
        # implementation as "no data" and "some data of zero length" respectively.

        successful :group {
            # The exchange was successful and a new session has been created for the authzid that
            # was established by the SASL exchange.

            session @3 :Session;
            # The session that was created. It grants access to all capabilities the connecting
            # party has permissions for.

            additionalData @4 :Data;
            # SASL may send additional data with the successful result. This MAY be a NULL-ptr or a
            # non-NULL list ptr of zero bytes which clients MUST pass to their SASL implementation
            # as "no additional data" and "some additional data of zero length" respectively.
        }
    }
}

interface Authentication {
    step @0 ( data: Data ) -> Response;
    # Respond to a challenge with more data. A client MUST NOT call this after having received an
    # "successful" response.

    abort @1 ();
    # Abort the current exchange. This will invalidate the Authentication making all further calls
    # to `step` return an error response. A client MUST NOT call this function after
    # having received an "successful" response.
    # A server will indicate that they have aborted an authentication exchange by replying with an
    # "aborted" Error to the next `step` call. A server SHOULD directly terminate the underlying stream
    # after sending this response. The server MAY after a short grace period terminate the stream
    # without sending a response if no call to `step` was received by the client.
}
