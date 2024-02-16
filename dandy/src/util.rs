use std::collections::HashSet;
use std::rc::Rc;

#[inline]
pub fn alphabet_equal(a: &[Rc<str>], b: &[Rc<str>]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let set1 = a.iter().collect::<HashSet<_>>();
    let set2 = b.iter().collect::<HashSet<_>>();
    set1 == set2
}
