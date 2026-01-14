// Easy typechecker tests - basic functionality

#[path = "common/mod.rs"]
mod common;
use common::*;
use typechecker::typecheck_prog;

#[test]
fn test_simple_int_variable() {
    // int x = 5;
    let prog = vec![d_gvar("x", e_int(5))];

    assert!(typecheck_prog(&prog).is_ok());
}

#[test]
fn test_simple_bool_variable() {
    // bool flag = true;
    let prog = vec![d_gvar("flag", e_bool(true))];

    assert!(typecheck_prog(&prog).is_ok());
}

#[test]
fn test_simple_function() {
    // int add(int a, int b) { return a; }
    let body = vec![s_ret(Some(e_id("a")))];
    let prog = vec![d_func(
        "add",
        vec![("a", t_int()), ("b", t_int())],
        ret_val(t_int()),
        body,
    )];

    assert!(typecheck_prog(&prog).is_ok());
}

#[test]
fn test_simple_arithmetic() {
    // int x = 5;
    // int y = 10;
    let prog = vec![d_gvar("x", e_int(5)), d_gvar("y", e_int(10))];

    assert!(typecheck_prog(&prog).is_ok());
}

#[test]
fn test_void_function() {
    // void foo() { return; }
    let body = vec![s_ret(None)];
    let prog = vec![d_func("foo", vec![], ret_void(), body)];

    assert!(typecheck_prog(&prog).is_ok());
}
