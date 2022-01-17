use std::collections::HashSet;
use std::hash::BuildHasher;
use std::hash::Hash;

#[allow(unused)]
/// Takes an arbitrary element from a `HashSet`, or None if empty.
pub fn hashset_take_arbitrary<K, S>(set: &mut HashSet<K, S>) -> Option<K>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    let key_ref = {
        if let Some(key_ref) = set.iter().next() {
            /* must hide the origin of this borrow ... */
            unsafe { &*(key_ref as *const _) }
        } else {
            return None;
        }
    };
    /* ... so that we may be able to mutably borrow the set here
    despite key_ref existence */
    set.take(key_ref)
}
