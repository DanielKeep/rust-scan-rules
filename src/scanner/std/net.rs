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
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use itertools::Itertools;
#[cfg(test)] use ::scanner::ScanFromStr;

parse_scanner! { impl<'a> for Ipv4Addr, matcher match_ipv4, matcher err "expected IPv4 address", err map ScanError::other }
parse_scanner! { impl<'a> for Ipv6Addr, matcher match_ipv6, matcher err "expected IPv6 address", err map ScanError::other }
parse_scanner! { impl<'a> for SocketAddr, matcher match_sock_addr, matcher err "expected socket address", err map ScanError::other }

fn match_ipv4(s: &str) -> Option<((usize, usize), usize)> {
    let ibs = &mut s.bytes().enumerate();
    try_opt!(eat_dec_digs(ibs));
    if !matches!(ibs.next(), Some((_, b'.'))) { return None; }
    try_opt!(eat_dec_digs(ibs));
    if !matches!(ibs.next(), Some((_, b'.'))) { return None; }
    try_opt!(eat_dec_digs(ibs));
    if !matches!(ibs.next(), Some((_, b'.'))) { return None; }
    eat_dec_digs(ibs)
}

fn match_ipv6(s: &str) -> Option<((usize, usize), usize)> {
    /*
        digraph ipv6 {
            START;
            Ok;
            Err;
        
            START -> 1 [label="\\x+"];
            START -> Err [label="*"];
            START -> "::" [label="::"];
            
            1 -> "1+" [label=":\\x+"];
            1 -> Err [label="*"];
        
            "1+" -> "1+" [label=":\\x+"];
            "1+" -> "::" [label="::"];
            "1+" -> Ok [label=":\\d+.\\d+.\\d+.\\d+"];
            "1+" -> Ok [label="*"];
        
            "::" -> "::+" [label="\\x+"];
            "::" -> Ok [label="\\d+.\\d+.\\d+.\\d+"];
            "::" -> Ok [label="*"];
        
            "::+" -> "::+" [label=":\\x+"];
            "::+" -> Ok [label=":\\d+.\\d+.\\d+.\\d+"];
            "::+" -> Ok [label="*"];
        }
    */
    fn eat_hex<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I) -> Option<((usize, usize), usize)> {
        let reset = ibs.clone();
        ibs.take_while_ref(|&(_, b)|
                matches!(b, b'0'...b'9' | b'a'...b'f' | b'A'...b'F'))
            .last()
            .map(|(i, _)| i + 1)
            .map(|n| ((0, n), n))
            .or_else(|| { *ibs = reset; None })
    }

    fn eat_dec<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I) -> Option<((usize, usize), usize)> {
        let reset = ibs.clone();
        ibs.take_while_ref(|&(_, b)|
                matches!(b, b'0'...b'9'))
            .last()
            .map(|(i, _)| i + 1)
            .map(|n| ((0, n), n))
            .or_else(|| { *ibs = reset; None })
    }

    fn eat_colon_hex<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I) -> Option<((usize, usize), usize)> {
        let reset = ibs.clone();
        (|| {
            if !matches!(ibs.next(), Some((_, b':'))) { return None; }
            eat_hex(ibs)
        })().or_else(|| { *ibs = reset; None })
    }

    fn eat_dbl_colon<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I) -> Option<((usize, usize), usize)> {
        let reset = ibs.clone();
        (|| {
            if !matches!(ibs.next(), Some((_, b':'))) { return None; }
            match ibs.next() {
                Some((i, b':')) => Some(((0, i + 1), i + 1)),
                _ => None,
            }
        })().or_else(|| { *ibs = reset; None })
    }

    fn eat_ipv4<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I) -> Option<((usize, usize), usize)> {
        let reset = ibs.clone();
        (|| {
            let _ = try_opt!(eat_dec(ibs));
            if !matches!(ibs.next(), Some((_, b'.'))) { return None; }
            let _ = try_opt!(eat_dec(ibs));
            if !matches!(ibs.next(), Some((_, b'.'))) { return None; }
            let _ = try_opt!(eat_dec(ibs));
            if !matches!(ibs.next(), Some((_, b'.'))) { return None; }
            eat_dec(ibs)
        })().or_else(|| { *ibs = reset; None })
    }

    fn eat_colon_ipv4<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I) -> Option<((usize, usize), usize)> {
        let reset = ibs.clone();
        (|| {
            if !matches!(ibs.next(), Some((_, b':'))) { return None; }
            eat_ipv4(ibs)
        })().or_else(|| { *ibs = reset; None })
    }

    fn start<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I) -> Option<((usize, usize), usize)> {
        if let Some(_) = eat_hex(ibs) {
            one(ibs)
        } else if let Some(end) = eat_dbl_colon(ibs) {
            dbl_colon(ibs, end)
        } else {
            None
        }
    }

    fn one<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I) -> Option<((usize, usize), usize)> {
        if let Some(end) = eat_colon_hex(ibs) {
            one_plus(ibs, end)
        } else {
            None
        }
    }

    fn one_plus<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I, mut end: ((usize, usize), usize)) -> Option<((usize, usize), usize)> {
        loop {
            if let Some(end) = eat_colon_ipv4(ibs) {
                return Some(end);
            } else if let Some(end) = eat_dbl_colon(ibs) {
                return dbl_colon(ibs, end);
            } else if let Some(new_end) = eat_colon_hex(ibs) {
                end = new_end;
                continue;
            } else {
                return Some(end);
            }
        }
    }

    fn dbl_colon<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I, end: ((usize, usize), usize)) -> Option<((usize, usize), usize)> {
        if let Some(end) = eat_ipv4(ibs) {
            Some(end)
        } else if let Some(end) = eat_hex(ibs) {
            dbl_colon_plus(ibs, end)
        } else {
            Some(end)
        }
    }

    fn dbl_colon_plus<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I, mut end: ((usize, usize), usize)) -> Option<((usize, usize), usize)> {
        loop {
            if let Some(end) = eat_colon_ipv4(ibs) {
                return Some(end);
            } else if let Some(new_end) = eat_colon_hex(ibs) {
                end = new_end;
                continue;
            } else {
                return Some(end);
            }
        }
    }

    let mut ibs = s.bytes().enumerate();
    match start(&mut ibs) {
        res => {
            res
        }
    }
}

fn match_sock_addr(s: &str) -> Option<((usize, usize), usize)> {
    match_ipv4_sock(s)
        .or_else(|| match_ipv6_sock(s))
}

fn match_ipv4_sock(s: &str) -> Option<((usize, usize), usize)> {
    let ((_, _), off) = try_opt!(match_ipv4(s));
    let mut ibs = s[off..].bytes().enumerate();
    if !matches!(ibs.next(), Some((_, b':'))) { return None; }
    eat_dec_digs(&mut ibs)
        .map(|((_, b), c)| ((0, b + off), c + off))
}

fn match_ipv6_sock(s: &str) -> Option<((usize, usize), usize)> {
    if !s.starts_with("[") { return None; }
    let ((_, _), off) = try_opt!(match_ipv6(&s[1..]));
    let off = off + 1;
    let mut ibs = s[off..].bytes().enumerate();
    if !matches!(ibs.next(), Some((_, b']'))) { return None; }
    if !matches!(ibs.next(), Some((_, b':'))) { return None; }
    eat_dec_digs(&mut ibs)
        .map(|((_, b), c)| ((0, b + off), c + off))
}

fn eat_dec_digs<I: Clone + Iterator<Item=(usize, u8)>>(ibs: &mut I) -> Option<((usize, usize), usize)> {
    ibs.take_while_ref(|&(_, b)| matches!(b, b'0'...b'9'))
        .last()
        .map(|(i, _)| i + 1)
        .map(|n| ((0, n), n))
}

#[cfg(test)]
#[test]
fn test_scan_ipv4addr() {
    use ::ScanError as SE;
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

    check_ipv4!("256.0.0.1"; Err(SE { kind: SEK::Other(_), .. }));
    check_ipv4!("255.0.0"; Err(SE { kind: SEK::Syntax(_), .. }));
    check_ipv4!("255.0.0.1.2"; Ok("255.0.0.1"));
    check_ipv4!("255.0..1"; Err(SE { kind: SEK::Syntax(_), .. }));
}

#[cfg(test)]
#[test]
fn test_scan_ipv6addr() {
    use ::ScanError as SE;
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

    check_ipv6!("::00000"; Err(SE { kind: SEK::Other(_), .. }));
    check_ipv6!("1:2:3:4:5:6:7"; Err(SE { kind: SEK::Other(_), .. }));
    check_ipv6!("1:2:3:4:5:6:7:8:9"; Err(SE { kind: SEK::Other(_), .. }));
    check_ipv6!("1:2:::6:7:8"; Ok("1:2::"));
    check_ipv6!("1:2::6::8"; Ok("1:2::6"));

    check_ipv6!("::192.0.2.33");
    check_ipv6!("::FFFF:192.0.2.33");
    check_ipv6!("64:ff9b::192.0.2.33");
    check_ipv6!("2001:db8:122:c000:2:2100:192.0.2.33");

    check_ipv6!("::127.0.0.1:"; Ok("::127.0.0.1"));
    check_ipv6!("1:2:3:4:5:127.0.0.1"; Err(SE { kind: SEK::Other(_), .. }));
    check_ipv6!("1:2:3:4:5:6:7:127.0.0.1"; Err(SE { kind: SEK::Other(_), .. }));
}

#[cfg(test)]
#[test]
fn test_scan_socketaddr() {
    use ::ScanError as SE;
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
    check_sockaddr!("255.255.255.255:65536"; Err(SE { kind: SEK::Other(_), .. }));

    check_sockaddr!("[0:0:0:0:0:0:0:0]:0");
    check_sockaddr!("[0:0:0:0:0:0:0:1]:0");
    check_sockaddr!("[::1]:0");
    check_sockaddr!("[::]:0");
    check_sockaddr!("[2a02:6b8::11:11]:0");
}

mod socket_addr_vx_scanners {
    use std::net::{SocketAddrV4, SocketAddrV6};
    use super::{match_ipv4_sock, match_ipv6_sock};
    #[cfg(test)] use ::scanner::ScanFromStr;

    parse_scanner! { impl<'a> for SocketAddrV4, matcher match_ipv4_sock, matcher err "expected IPv4 socket address", err map ScanError::other }
    parse_scanner! { impl<'a> for SocketAddrV6, matcher match_ipv6_sock, matcher err "expected IPv6 socket address", err map ScanError::other }

    #[cfg(test)]
    #[test]
    fn test_scan_socketaddrv4() {
        use ::ScanError as SE;
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
        check_ipv4!("255.255.255.255:65536"; Err(SE { kind: SEK::Other(_), .. }));
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
}
