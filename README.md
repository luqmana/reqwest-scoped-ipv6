# Scoped IPv6 Addresses in URLs

[reqwest](https://github.com/seanmonstar/reqwest) unfortunately does not play nice with URLs that attempt to use scoped IPv6 addresses<sup>1</sup>.

This is not a new class of problem, UNC paths on Windows don't support IPv6 as colons are not a valid character in paths. The workaround then? [Encode the address as a special domain](https://devblogs.microsoft.com/oldnewthing/20100915-00/?p=12863):
>Take your IPv6 address, replace the colons with dashes, replace percent signs with the letter “s”, and append .ipv6-literal.net.

Taking that same idea and reqwest's overridable DNS support we can do the same.


<sup>1</sup> Really anything using the [url](https://github.com/servo/rust-url) crate [[issue](https://github.com/servo/rust-url/issues/424)].