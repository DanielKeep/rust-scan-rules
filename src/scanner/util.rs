use regex::Regex;

lazy_static! {
    static ref WORD_RE: Regex = Regex::new(r"^\s*(\w+|\S)").unwrap();
}

pub trait StrUtil {
    fn split_word(&self) -> Option<(&Self, &Self)>;

    /**
    Returns the byte offset of an inner slice relative to an enclosing outer slice.
    */
    fn subslice_offset(&self, inner: &Self) -> Option<usize>;
}

impl StrUtil for str {
    fn split_word(&self) -> Option<(&Self, &Self)> {
        WORD_RE.find(self)
            .map(|(a, b)| (&self[a..b], &self[b..]))
    }

    fn subslice_offset(&self, inner: &str) -> Option<usize> {
        let self_beg = self.as_ptr() as usize;
        let inner = inner.as_ptr() as usize;
        if inner < self_beg || inner > self_beg.wrapping_add(self.len()) {
            None
        } else {
            Some(inner.wrapping_sub(self_beg))
        }
    }
}

#[test]
fn test_subslice_offset() {
    let string = "a\nb\nc";
    let lines: Vec<&str> = string.lines().collect();

    assert!(string.subslice_offset(lines[0]) == Some(0)); // &"a"
    assert!(string.subslice_offset(lines[1]) == Some(2)); // &"b"
    assert!(string.subslice_offset(lines[2]) == Some(4)); // &"c"
    assert!(string.subslice_offset("other!") == None);
}
