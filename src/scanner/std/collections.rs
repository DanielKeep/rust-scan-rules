use std::collections::{
    BTreeMap, BTreeSet, BinaryHeap,
    HashMap, HashSet,
    LinkedList,
    VecDeque,
};
use std::hash::Hash;
use ::scanner::KeyValuePair;

scanner! { impl<'a, K, V> ScanFromStr for BTreeMap<K, V> where {K: Ord} {
    ("{", [ let es: KeyValuePair<K, V> ],*: BTreeMap<K, V>, "}", ..tail) => (es, tail)
}}

scanner! { impl<'a, T> ScanFromStr for BTreeSet<T> where {T: Ord} {
    ("{", [ let es: T ],*: BTreeSet<_>, "}", ..tail) => (es, tail)
}}

scanner! { impl<'a, T> ScanFromStr for BinaryHeap<T> where {T: Ord} {
    ("[", [ let es: T ],*: BinaryHeap<_>, "]", ..tail) => (es, tail)
}}

scanner! { impl<'a, K, V> ScanFromStr for HashMap<K, V> where {K: Hash + Eq} {
    ("{", [ let es: KeyValuePair<K, V> ],*: HashMap<K, V>, "}", ..tail) => (es, tail)
}}

scanner! { impl<'a, T> ScanFromStr for HashSet<T> where {T: Hash + Eq} {
    ("{", [ let es: T ],*: HashSet<_>, "}", ..tail) => (es, tail)
}}

scanner! { impl<'a, T> ScanFromStr for LinkedList<T> {
    ("[", [ let es: T ],*: LinkedList<_>, "]", ..tail) => (es, tail)
}}

scanner! { impl<'a, T> ScanFromStr for Vec<T> {
    ("[", [ let es: T ],*, "]", ..tail) => (es, tail)
}}

scanner! { impl<'a, T> ScanFromStr for VecDeque<T> {
    ("[", [ let es: T ],*: VecDeque<_>, "]", ..tail) => (es, tail)
}}

#[cfg(test)]
#[test]
fn test_btreemap() {
    use ::ScanErrorKind as SEK;
    use ::scanner::ScanFromStr;

    macro_rules! check {
        (<$k:ty, $v:ty> $s:expr, Ok($r:expr, $n:expr)) => {
            assert_match!(
                <BTreeMap<$k, $v>>::scan_from($s),
                Ok((ref v, $n)) if &*sorted(v.clone().into_iter()) == &$r
            )
        };

        (<$k:ty, $v:ty> $s:expr, Err($err:pat)) => {
            assert_match!(
                <BTreeMap<$k, $v>>::scan_self_from($s),
                Err($err)
            )
        };
    }

    check!(<i32, i32> "{}", Ok([], 2));
    check!(<i32, i32> "{0: 1}", Ok([(0, 1)], 6));
    check!(<i32, i32> "{0: 1, 2: 3}", Ok([(0, 1), (2, 3)], 12));
    check!(<i32, bool> "{0: true, 1: false}", Ok([(0, true), (1, false)], 19));
}

#[cfg(test)]
#[test]
fn test_btreeset() {
    use ::ScanErrorKind as SEK;
    use ::scanner::ScanFromStr;

    macro_rules! check {
        (<$ty:ty> $s:expr, Ok($v:expr, $n:expr)) => {
            assert_match!(
                <BTreeSet<$ty>>::scan_from($s),
                Ok((ref v, $n)) if &*sorted(v.into_iter().cloned()) == &$v
            )
        };

        (<$ty:ty> $s:expr, Err($err:pat)) => {
            assert_match!(
                <BTreeSet<$ty>>::scan_self_from($s),
                Err($err)
            )
        };
    }

    check!(<i32> "{}", Ok([], 2));
    check!(<i32> "{0}", Ok([0], 3));
    check!(<i32> "{0, 1}", Ok([0, 1], 6));
    check!(<bool> "{true, false}", Ok([false, true], 13));
}

#[cfg(test)]
#[test]
fn test_binaryheap() {
    use ::ScanErrorKind as SEK;
    use ::scanner::ScanFromStr;

    macro_rules! check {
        (<$ty:ty> $s:expr, Ok($v:expr, $n:expr)) => {
            assert_match!(
                <BinaryHeap<$ty>>::scan_from($s),
                Ok((ref v, $n)) if &*sorted(v.into_iter().cloned()) == &$v
            )
        };

        (<$ty:ty> $s:expr, Err($err:pat)) => {
            assert_match!(
                <BinaryHeap<$ty>>::scan_self_from($s),
                Err($err)
            )
        };
    }

    check!(<i32> "[]", Ok([], 2));
    check!(<i32> "[0]", Ok([0], 3));
    check!(<i32> "[0, 1]", Ok([0, 1], 6));
    check!(<bool> "[true, false]", Ok([false, true], 13));
}

#[cfg(test)]
#[test]
fn test_hashmap() {
    use ::ScanErrorKind as SEK;
    use ::scanner::ScanFromStr;

    macro_rules! check {
        (<$k:ty, $v:ty> $s:expr, Ok($r:expr, $n:expr)) => {
            assert_match!(
                <HashMap<$k, $v>>::scan_from($s),
                Ok((ref v, $n)) if &*sorted(v.clone().into_iter()) == &$r
            )
        };

        (<$k:ty, $v:ty> $s:expr, Err($err:pat)) => {
            assert_match!(
                <HashMap<$k, $v>>::scan_self_from($s),
                Err($err)
            )
        };
    }

    check!(<i32, i32> "{}", Ok([], 2));
    check!(<i32, i32> "{0: 1}", Ok([(0, 1)], 6));
    check!(<i32, i32> "{0: 1, 2: 3}", Ok([(0, 1), (2, 3)], 12));
    check!(<i32, bool> "{0: true, 1: false}", Ok([(0, true), (1, false)], 19));
}

#[cfg(test)]
#[test]
fn test_hashset() {
    use ::ScanErrorKind as SEK;
    use ::scanner::ScanFromStr;

    macro_rules! check {
        (<$ty:ty> $s:expr, Ok($v:expr, $n:expr)) => {
            assert_match!(
                <HashSet<$ty>>::scan_from($s),
                Ok((ref v, $n)) if &*sorted(v.into_iter().cloned()) == &$v
            )
        };

        (<$ty:ty> $s:expr, Err($err:pat)) => {
            assert_match!(
                <HashSet<$ty>>::scan_self_from($s),
                Err($err)
            )
        };
    }

    check!(<i32> "{}", Ok([], 2));
    check!(<i32> "{0}", Ok([0], 3));
    check!(<i32> "{0, 1}", Ok([0, 1], 6));
    check!(<bool> "{true, false}", Ok([false, true], 13));
}

#[cfg(test)]
#[test]
fn test_linkedlist() {
    use ::ScanErrorKind as SEK;
    use ::scanner::ScanFromStr;

    macro_rules! check {
        (<$ty:ty> $s:expr, Ok($v:expr, $n:expr)) => {
            assert_match!(
                <LinkedList<$ty>>::scan_from($s),
                Ok((ref v, $n)) if &*(v.clone().into_iter().collect::<Vec<_>>()) == &$v
            )
        };

        (<$ty:ty> $s:expr, Err($err:pat)) => {
            assert_match!(
                <LinkedList<$ty>>::scan_self_from($s),
                Err($err)
            )
        };
    }

    check!(<i32> "[]", Ok([], 2));
    check!(<i32> "[0]", Ok([0], 3));
    check!(<i32> "[0, 1]", Ok([0, 1], 6));
    check!(<bool> "[true, false]", Ok([true, false], 13));
}

#[cfg(test)]
#[test]
fn test_vec() {
    use ::ScanErrorKind as SEK;
    use ::scanner::ScanFromStr;

    macro_rules! check {
        (<$ty:ty> $s:expr, Ok($v:expr, $n:expr)) => {
            assert_match!(
                <Vec<$ty>>::scan_from($s),
                Ok((ref v, $n)) if &*v == &$v
            )
        };

        (<$ty:ty> $s:expr, Err($err:pat)) => {
            assert_match!(
                <Vec<$ty>>::scan_self_from($s),
                Err($err)
            )
        };
    }

    check!(<i32> "[]", Ok([], 2));
    check!(<i32> "[0]", Ok([0], 3));
    check!(<i32> "[0, 1]", Ok([0, 1], 6));
    check!(<bool> "[true, false]", Ok([true, false], 13));
}

#[cfg(test)]
#[test]
fn test_vecdeque() {
    use ::ScanErrorKind as SEK;
    use ::scanner::ScanFromStr;

    macro_rules! check {
        (<$ty:ty> $s:expr, Ok($v:expr, $n:expr)) => {
            assert_match!(
                <VecDeque<$ty>>::scan_from($s),
                Ok((ref v, $n)) if &*(v.clone().into_iter().collect::<Vec<_>>()) == &$v
            )
        };

        (<$ty:ty> $s:expr, Err($err:pat)) => {
            assert_match!(
                <VecDeque<$ty>>::scan_self_from($s),
                Err($err)
            )
        };
    }

    check!(<i32> "[]", Ok([], 2));
    check!(<i32> "[0]", Ok([0], 3));
    check!(<i32> "[0, 1]", Ok([0, 1], 6));
    check!(<bool> "[true, false]", Ok([true, false], 13));
}

#[cfg(test)]
fn sorted<It: Iterator>(it: It) -> Vec<It::Item>
where It::Item: Ord {
    let mut v: Vec<_> = it.collect();
    v.sort();
    v
}
