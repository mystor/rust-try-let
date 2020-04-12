#![feature(proc_macro_hygiene)]
#![feature(bindings_after_at)]

use try_let::try_let;

#[test]
fn bindings_after_at() {
    let nested = Some(("apple", "pear"));

    try_let!(Some(a @ (b, c)) = nested else {
        unreachable!();
    });
    assert_eq!(a, ("apple", "pear"));
    assert_eq!(b, "apple");
    assert_eq!(c, "pear");
}
