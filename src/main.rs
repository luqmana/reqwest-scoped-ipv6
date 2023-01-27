#[cfg(unix)]
mod unix {
    use std::{
        io,
        net::{Ipv6Addr, SocketAddrV6},
    };

    use futures_util::future::{self, FutureExt};
    use hyper::{
        client::connect::dns::{GaiResolver, Name},
        service::Service,
    };
    use libc::if_nametoindex;
    use reqwest::dns::{Resolve, Resolving};

    pub struct Ipv6LiteralResolver;

    impl Resolve for Ipv6LiteralResolver {
        fn resolve(&self, name: Name) -> Resolving {
            if name.as_str().ends_with(".ipv6-literal.net") {
                let addr = name
                    .as_str()
                    .trim_end_matches(".ipv6-literal.net")
                    .replace('-', ":");

                // Split the address and the scope id (if any)
                let mut parts = addr.splitn(2, 's');

                let ip6addr = match parts.next() {
                    Some(addr_part) => match addr_part.parse::<Ipv6Addr>() {
                        Ok(ip6addr) => ip6addr,
                        Err(err) => {
                            let err = io::Error::new(io::ErrorKind::InvalidInput, err);
                            return Box::pin(future::err(Box::new(err) as _));
                        }
                    },
                    None => {
                        let err = io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "ipv6-literal.net address must contain an IPv6 address",
                        );
                        return Box::pin(future::err(Box::new(err) as _));
                    }
                };

                let scope_id = match parts.next() {
                    Some(s) => match s.parse::<u32>() {
                        Ok(scope_id) => scope_id,
                        Err(_) if s.is_ascii() => {
                            // Not numeric, try to parse as interface name
                            let name = match std::ffi::CString::new(s) {
                                Ok(name) => name,
                                Err(_) => {
                                    let err = io::Error::new(
                                        io::ErrorKind::InvalidInput,
                                        "invalid IPv6 zone identifier (interior NUL)",
                                    );
                                    return Box::pin(future::err(Box::new(err) as _));
                                }
                            };
                            match unsafe { if_nametoindex(name.as_ptr()) } {
                                0 => {
                                    let err = io::Error::new(
                                        io::ErrorKind::InvalidInput,
                                        "invalid IPv6 zone identifier",
                                    );
                                    return Box::pin(future::err(Box::new(err) as _));
                                }
                                index => index,
                            }
                        }
                        Err(_) => {
                            let err = io::Error::new(
                                io::ErrorKind::InvalidInput,
                                "invalid IPv6 zone identifier",
                            );
                            return Box::pin(future::err(Box::new(err) as _));
                        }
                    },
                    None => 0,
                };

                // The port here doesn't matter, reqwest will overwrite it with either the default port
                // (e.g. 80, 443) or the port specified in the URL.
                let addr6 = SocketAddrV6::new(ip6addr, 0, 0, scope_id);

                return Box::pin(future::ok(Box::new(std::iter::once(addr6.into())) as _));
            }

            // Otherwise fallback to standard getaddrinfo-based resolver
            Box::pin(
                Service::<Name>::call(&mut GaiResolver::new(), name).map(|gai_res| {
                    gai_res
                        .map(|addrs| Box::new(addrs) as _)
                        .map_err(|err| Box::new(err) as _)
                }),
            )
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(unix)]
    let client = reqwest::Client::builder()
        .dns_resolver(std::sync::Arc::new(unix::Ipv6LiteralResolver))
        .build()?;
    #[cfg(windows)]
    let client = reqwest::Client::new();

    let res = client
        .get("http://fe80--93aa-4223-e7a4-9975sen0.ipv6-literal.net:8888/")
        .send()
        .await?
        .text()
        .await?;

    println!("{res}");

    Ok(())
}
