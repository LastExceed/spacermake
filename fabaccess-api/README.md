# FabAccess API

## Code generation bugs under c#
When returning an Interface it may be required to append a dummy valueto work around a c# code generation bug.

```diff
- whoami @4 () -> ( you :Api.User );
+ whoami @4 () -> ( you :Api.User, dummy :UInt8 = 0  );
```

## Docs
A lot of information (concepts, usage, decisions) about this API can be found at [docs.fab-access.org](https://docs.fab-access.org/books/schnittstellen-und-apis/page/fabaccess-api#bkmrk-fabaccess-api).

See also:
- [pyfabapi (Python Wrapper)](https://gitlab.com/fabinfra/fabaccess/pyfabapi)
- [FabAccess-API-cs (C# implementation)](https://gitlab.com/fabinfra/fabaccess/fabaccess-api-cs)