#![feature(proc_macro_hygiene)]

use try_let::try_let;

#[test]
fn basic() {
    let foo: Option<i32> = Some(10);
    try_let!(Some(x) = foo else unreachable!());
    assert_eq!(x, 10);

    try_let!(None = foo else return);
    unreachable!();
}

#[test]
fn tuple() {
    let other = (Some(10), Some("apples"), None::<()>);
    try_let!((Some(a), Some(b), None) = other else unreachable!());

    assert_eq!(a, 10);
    assert_eq!(b, "apples");
}
