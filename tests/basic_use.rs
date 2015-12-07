#![feature(plugin)]

#![plugin(try_let)]

#[test]
fn else_expr_form() {
    let foo: Option<i32> = Some(10);
    try_let!(Some(x) = foo else unreachable!());
    assert_eq!(x, 10);

    try_let!(None = foo else return);
    unreachable!();
}

#[test]
fn match_block_form() {
    let foo: Result<i32, i32> = Ok(20);
    try_let!(Ok(y) = foo {
        _ => unreachable!()
    });
    assert_eq!(y, 20);

    try_let!(z @ Ok(20) = foo {
        _ => unreachable!()
    });
    assert_eq!(z, Ok(20));

    let foo: Result<i32, i32> = Err(30);
    try_let!(Ok(x) = foo {
        Err(30) => return,
        _ => unreachable!()
    });
    assert_eq!(x, -1); // To supress unused variable warnings
    unreachable!();
}

