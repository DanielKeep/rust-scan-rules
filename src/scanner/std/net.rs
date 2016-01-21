/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Scanner implementations for `std::net::*`.
*/
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use regex::Regex;
#[cfg(test)] use ::scanner::ScanFromStr;

macro_rules! as_expr { ($e:expr) => { $e } }

macro_rules! addr_regexen {
    (
        ipv4: $ipv4:tt,
        ipv6: $ipv6:tt,
        sad4: ($sad4a:tt, ipv4, $sad4b:tt),
        sad6: ($sad6a:tt, ipv6, $sad6b:tt),
    ) => {
        lazy_static! {
            static ref IPV4ADDR_RE: Regex = Regex::new(
                as_expr!(concat!("(?x)^", $ipv4))
            ).unwrap();

            static ref IPV6ADDR_RE: Regex = Regex::new(
                as_expr!(concat!("(?x)^", $ipv6))
            ).unwrap();

            static ref SOCKADDRV4_RE: Regex = Regex::new(
                as_expr!(concat!("(?x)^", $sad4a, $ipv4, $sad4b))
            ).unwrap();

            static ref SOCKADDRV6_RE: Regex = Regex::new(
                as_expr!(concat!("(?x)^", $sad6a, $ipv6, $sad6b))
            ).unwrap();

            static ref SOCKADDR_RE: Regex = Regex::new(
                as_expr!(concat!("(?x)^(", $sad4a, $ipv4, $sad4b, ")|(", $sad6a, $ipv6, $sad6b, ")"))
            ).unwrap();
        }
    };
}

addr_regexen! {
    ipv4: r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}",
    ipv6: r"
          (
            ( [:xdigit:]+ (:[:xdigit:]+)* )?
            ::
            ( [:xdigit:]+ (:[:xdigit:]+)* (\.\d+\.\d+\.\d+)? )?
        )
        | [:xdigit:]+ (:[:xdigit:]+)+ (\.\d+\.\d+\.\d+)?
    ",
    sad4: (r"(", ipv4, r"):\d+"),
    sad6: (r"\[(", ipv6, r")\]:\d+"),
}

parse_scanner! { impl<'a> for Ipv4Addr, regex IPV4ADDR_RE, regex err "expected IPv4 address", err map ScanErrorKind::from_other }
parse_scanner! { impl<'a> for Ipv6Addr, regex IPV6ADDR_RE, regex err "expected IPv6 address", err map ScanErrorKind::from_other }
parse_scanner! { impl<'a> for SocketAddr, regex SOCKADDR_RE, regex err "expected socket address", err map ScanErrorKind::from_other }
parse_scanner! { impl<'a> for SocketAddrV4, regex SOCKADDRV4_RE, regex err "expected IPv4 socket address", err map ScanErrorKind::from_other }
parse_scanner! { impl<'a> for SocketAddrV6, regex SOCKADDRV6_RE, regex err "expected IPv6 socket address", err map ScanErrorKind::from_other }

#[cfg(test)]
#[test]
fn test_scan_ipv4addr() {
    use ::ScanErrorKind as SEK;

    macro_rules! check_ipv4 {
        ($s:expr) => {
            assert_match!(
                <Ipv4Addr>::scan_from($s),
                Ok((v, n)) if v == $s.parse().unwrap() && n == $s.len()
            )
        };

        ($s:expr; Ok($v:expr)) => {
            assert_match!(
                <Ipv4Addr>::scan_from($s),
                Ok((v, n)) if v == $v.parse().unwrap() && n == $v.len()
            )
        };

        ($s:expr; Err($err:pat)) => {
            assert_match!(
                <Ipv4Addr>::scan_from($s),
                Err($err)
            )
        };
    }

    check_ipv4!("0.0.0.0");
    check_ipv4!("127.0.0.1");
    check_ipv4!("255.255.255.255");

    check_ipv4!("256.0.0.1"; Err(SEK::Other(_)));
    check_ipv4!("255.0.0"; Err(SEK::Syntax(_)));
    check_ipv4!("255.0.0.1.2"; Ok("255.0.0.1"));
    check_ipv4!("255.0..1"; Err(SEK::Syntax(_)));
}

#[cfg(test)]
#[test]
fn test_scan_ipv6addr() {
    use ::ScanErrorKind as SEK;

    macro_rules! check_ipv6 {
        ($s:expr) => {
            assert_match!(
                <Ipv6Addr>::scan_from($s),
                Ok((v, n)) if v == $s.parse().unwrap() && n == $s.len()
            )
        };

        ($s:expr; Ok($v:expr)) => {
            assert_match!(
                <Ipv6Addr>::scan_from($s),
                Ok((v, n)) if v == $v.parse().unwrap() && n == $v.len()
            )
        };

        ($s:expr; Err($err:pat)) => {
            assert_match!(
                <Ipv6Addr>::scan_from($s),
                Err($err)
            )
        };
    }

    check_ipv6!("0:0:0:0:0:0:0:0");
    check_ipv6!("0:0:0:0:0:0:0:1");
    check_ipv6!("::1");
    check_ipv6!("::");
    check_ipv6!("2a02:6b8::11:11");

    check_ipv6!("::00000"; Err(SEK::Other(_)));
    check_ipv6!("1:2:3:4:5:6:7"; Err(SEK::Other(_)));
    check_ipv6!("1:2:3:4:5:6:7:8:9"; Err(SEK::Other(_)));
    check_ipv6!("1:2:::6:7:8"; Ok("1:2::"));
    check_ipv6!("1:2::6::8"; Ok("1:2::6"));

    check_ipv6!("::192.0.2.33");
    check_ipv6!("::FFFF:192.0.2.33");
    check_ipv6!("64:ff9b::192.0.2.33");
    check_ipv6!("2001:db8:122:c000:2:2100:192.0.2.33");

    check_ipv6!("::127.0.0.1:"; Ok("::127.0.0.1"));
    check_ipv6!("1:2:3:4:5:127.0.0.1"; Err(SEK::Other(_)));
    check_ipv6!("1:2:3:4:5:6:7:127.0.0.1"; Err(SEK::Other(_)));
}

#[cfg(test)]
#[test]
fn test_scan_socketaddr() {
    use ::ScanErrorKind as SEK;

    macro_rules! check_sockaddr {
        ($s:expr) => {
            assert_match!(
                <SocketAddr>::scan_from($s),
                Ok((v, n)) if v == $s.parse().unwrap() && n == $s.len()
            )
        };

        ($s:expr; Ok($v:expr)) => {
            assert_match!(
                <SocketAddr>::scan_from($s),
                Ok((v, n)) if v == $v.parse().unwrap() && n == $v.len()
            )
        };

        ($s:expr; Err($err:pat)) => {
            assert_match!(
                <SocketAddr>::scan_from($s),
                Err($err)
            )
        };
    }

    check_sockaddr!("0.0.0.0:0");
    check_sockaddr!("127.0.0.1:80");
    check_sockaddr!("255.255.255.255:65535");
    check_sockaddr!("255.255.255.255:65536"; Err(SEK::Other(_)));

    check_sockaddr!("[0:0:0:0:0:0:0:0]:0");
    check_sockaddr!("[0:0:0:0:0:0:0:1]:0");
    check_sockaddr!("[::1]:0");
    check_sockaddr!("[::]:0");
    check_sockaddr!("[2a02:6b8::11:11]:0");
}

#[cfg(test)]
#[test]
fn test_scan_socketaddrv4() {
    use ::ScanErrorKind as SEK;

    macro_rules! check_ipv4 {
        ($s:expr) => {
            assert_match!(
                <SocketAddrV4>::scan_from($s),
                Ok((v, n)) if v == $s.parse().unwrap() && n == $s.len()
            )
        };

        ($s:expr; Ok($v:expr)) => {
            assert_match!(
                <SocketAddrV4>::scan_from($s),
                Ok((v, n)) if v == $v.parse().unwrap() && n == $v.len()
            )
        };

        ($s:expr; Err($err:pat)) => {
            assert_match!(
                <SocketAddrV4>::scan_from($s),
                Err($err)
            )
        };
    }

    check_ipv4!("0.0.0.0:0");
    check_ipv4!("127.0.0.1:80");
    check_ipv4!("255.255.255.255:65535");
    check_ipv4!("255.255.255.255:65536"; Err(SEK::Other(_)));
}

#[cfg(test)]
#[test]
fn test_scan_socketaddrv6() {
    macro_rules! check_ipv6 {
        ($s:expr) => {
            assert_match!(
                <SocketAddrV6>::scan_from($s),
                Ok((v, n)) if v == $s.parse().unwrap() && n == $s.len()
            )
        };

        ($s:expr; Ok($v:expr)) => {
            assert_match!(
                <SocketAddrV6>::scan_from($s),
                Ok((v, n)) if v == $v.parse().unwrap() && n == $v.len()
            )
        };

        ($s:expr; Err($err:pat)) => {
            assert_match!(
                <SocketAddrV6>::scan_from($s),
                Err($err)
            )
        };
    }

    check_ipv6!("[0:0:0:0:0:0:0:0]:0");
    check_ipv6!("[0:0:0:0:0:0:0:1]:0");
    check_ipv6!("[::1]:0");
    check_ipv6!("[::]:0");
    check_ipv6!("[2a02:6b8::11:11]:0");
}
