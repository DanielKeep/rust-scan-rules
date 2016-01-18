use itertools::Itertools;
use regex::Regex;
use ::{ScanError, ScanErrorKind};

lazy_static! {
    static ref LITERAL_PART_RE: Regex = Regex::new(r"(\w+|\S)").unwrap();
}

pub trait ScanInput<'a>: Sized {
    fn try_end(self) -> Result<(), (ScanError<'a>, Self)>;

    fn try_scan<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError<'a>, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind>;

    fn try_scan_raw<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError<'a>, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind>;

    fn try_match_literal(self, lit: &str) -> Result<Self, (ScanError<'a>, Self)>;
}

impl<'a> ScanInput<'a> for &'a str {
    fn try_end(self) -> Result<(), (ScanError<'a>, Self)> {
        if self.len() == 0 {
            Ok(())
        } else {
            Err((ScanError::unexpected_end(self), self))
        }
    }

    fn try_scan<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError<'a>, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind> {
        let tmp = skip_space(self);
        match f(tmp) {
            Ok((out, off)) => Ok((out, &tmp[off..])),
            Err(err) => Err((ScanError::new(tmp, err), self))
        }
    }

    fn try_scan_raw<F, Out>(self, f: F) -> Result<(Out, Self), (ScanError<'a>, Self)>
    where F: FnOnce(&'a str) -> Result<(Out, usize), ScanErrorKind> {
        match f(self) {
            Ok((out, off)) => Ok((out, &self[off..])),
            Err(err) => Err((ScanError::new(self, err), self))
        }
    }

    fn try_match_literal(self, lit: &str) -> Result<Self, (ScanError<'a>, Self)> {
        use itertools::EitherOrBoth::{Both, Left};
        let tmp = skip_space(self);
        let lit = skip_space(lit);
        let inp_parts = LITERAL_PART_RE.find_iter(tmp);
        let lit_parts = LITERAL_PART_RE.find_iter(lit);
        let mut last_pos = 0;
        for ilp in inp_parts.zip_longest(lit_parts) {
            let (i1, ip, lp) = match ilp {
                Both((i0, i1), (l0, l1)) => (i1, &tmp[i0..i1], &lit[l0..l1]),
                Left(_) => break,
                _ => return Err((ScanError::literal_mismatch(self), self))
            };
            if ip != lp {
                return Err((ScanError::literal_mismatch(self), self));
            }
            last_pos = i1;
        }
        Ok(&tmp[last_pos..])
    }
}

fn skip_space(s: &str) -> &str {
    let off = s.char_indices()
        .take_while(|&(_, c)| c.is_whitespace())
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);
    &s[off..]
}
