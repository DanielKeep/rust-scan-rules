/*
Copyright ⓒ 2016 Daniel Keep.

Licensed under the MIT license (see LICENSE or <http://opensource.org
/licenses/MIT>) or the Apache License, Version 2.0 (see LICENSE of
<http://www.apache.org/licenses/LICENSE-2.0>), at your option. All
files in the project carrying such notice may not be copied, modified,
or distributed except according to those terms.
*/
/*!
Demonstrates parsing the contents of `/proc/$PID/maps`.
*/
#[macro_use] extern crate bitflags;
#[macro_use] extern crate scan_rules;
#[macro_use] mod util;

use std::ops::Range;
use scan_rules::ScanError;
use scan_rules::input::{ScanCursor, ScanInput};
use scan_rules::scanner::ScanFromStr;
use scan_rules::scanner::Hex;

const MAP_FILE: &'static str = include_str!("data/maps");

#[derive(Debug)]
struct Entry<'a> {
    pub range: Range<u64>,
    pub perm: Permissions,
    pub offset: u64,
    pub dev: Device,
    pub inode: Option<u64>,
    pub pathname: Option<&'a str>,
}

bitflags! {
    flags Permissions: u8 {
        const PERM_R = 0b1000,
        const PERM_W = 0b0100,
        const PERM_X = 0b0010,
        const PERM_S = 0b0001,
    }
}

impl<'a> ScanFromStr<'a> for Permissions {
    type Output = Self;

    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        let bs = s.as_str().as_bytes();

        if bs.len() < 4 {
            return Err(ScanError::syntax("expected permissions"));
        }

        let mut r = Permissions::empty();

        match bs[0] {
            b'r' => r = r | PERM_R,
            b'-' => (),
            _ => return Err(ScanError::syntax("expected `r` or `-`")),
        }
        match bs[1] {
            b'w' => r = r | PERM_W,
            b'-' => (),
            _ => return Err(ScanError::syntax("expected `w` or `-`")),
        }
        match bs[2] {
            b'x' => r = r | PERM_X,
            b'-' => (),
            _ => return Err(ScanError::syntax("expected `x` or `-`")),
        }
        match bs[3] {
            b's' => r = r | PERM_S,
            b'p' => (),
            _ => return Err(ScanError::syntax("expected `p` or `s`")),
        }

        Ok((r, 4))
    }
}

#[derive(Debug)]
struct Device(u8, u8);

impl<'a> ScanFromStr<'a> for Device {
    type Output = Self;

    fn scan_from<I: ScanInput<'a>>(s: I) -> Result<(Self::Output, usize), ScanError> {
        scan!(s.to_cursor();
            (let major: Hex<u8>, ":", let minor: Hex<u8>, ^..tail) => {
                (Device(major, minor), tail.offset())
            }
        )
    }
}

#[test]
fn test_maps() {
    let entries: Vec<_> = MAP_FILE.lines()
        .map(|line| scan!(line;
            (
                let min: Hex<u64>, "-", let max: Hex<u64>,
                let perm: Permissions,
                let offset: Hex<u64>,
                let dev: Device,
                let inode: u64,
                ..tail
            ) => {
                let inode = if inode == 0 {
                    None
                } else {
                    Some(inode)
                };

                let pathname = tail.trim();
                let pathname = if pathname == "" {
                    None
                } else {
                    Some(pathname)
                };

                Entry {
                    range: min..max,
                    perm: perm,
                    offset: offset,
                    dev: dev,
                    inode: inode,
                    pathname: pathname,
                }
            }
        ))
        .filter_map(|e| {
            // Throw away entries that didn't parse.
            e.ok()
        })
        .collect();

    assert_eq!(entries.len(), 49);

    // [0]: 00400000-004ef000 r-xp 00000000 08:01 1572870 /bin/bash
    assert_match!(
        &entries[0],
        &Entry {
            range: Range { start: 0x400000, end: 0x4ef000 },
            perm: Permissions { bits: 0b1010 },
            offset: 0x00000000,
            dev: Device(0x08, 0x01),
            inode: Some(1572870),
            pathname: Some("/bin/bash"),
        }
    );

    // [3]: 006f9000-006ff000 rw-p 00000000 00:00 0
    assert_match!(
        &entries[3],
        &Entry {
            range: Range { start: 0x6f9000, end: 0x6ff000 },
            perm: Permissions { bits: 0b1100 },
            offset: 0x00000000,
            dev: Device(0x00, 0x00),
            inode: None,
            pathname: None,
        }
    );

    // [33]: 7fba16472000-7fba16671000 ---p 00025000 08:01 2888164 /lib/x86_64-linux-gnu/libtinfo.so.5.9
    assert_match!(
        &entries[33],
        &Entry {
            range: Range { start: 0x7fba16472000, end: 0x7fba16671000 },
            perm: Permissions { bits: 0b0000 },
            offset: 0x00025000,
            dev: Device(0x08, 0x01),
            inode: Some(2888164),
            pathname: Some("/lib/x86_64-linux-gnu/libtinfo.so.5.9"),
        }
    );
}
