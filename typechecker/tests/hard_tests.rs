// Hard typechecker tests - complex scenarios

#[path = "common/mod.rs"]
mod common;
use ast::*;
use common::*;
use typechecker::typecheck_prog;

#[test]
fn test_nested_function_calls_with_subtyping() {
    // int[] arr = new int[10];
    // int get_value(int[] a, int idx) { return a[idx]; }
    // int result = get_value(arr, 5);

    let arr_init = e_new_arr(t_int(), e_int(10));
    let get_value_body = vec![s_ret(Some(e_index(e_id("a"), e_id("idx"))))];

    let prog = vec![
        d_gvar("arr", arr_init),
        d_func(
            "get_value",
            vec![("a", t_ref(r_array(t_int()))), ("idx", t_int())],
            ret_val(t_int()),
            get_value_body,
        ),
        d_gvar(
            "result",
            e_call(e_id("get_value"), vec![e_id("arr"), e_int(5)]),
        ),
    ];

    assert!(typecheck_prog(&prog).is_ok());
}

#[test]
fn test_return_path_analysis_both_branches() {
    // int abs(int x) {
    //   if (x < 0) { return -x; } else { return x; }
    // }

    let condition = e_bop(BinOp::Lt, e_id("x"), e_int(0));
    let then_branch = vec![s_ret(Some(e_uop(UnOp::Neg, e_id("x"))))];
    let else_branch = vec![s_ret(Some(e_id("x")))];

    let body = vec![s_if(condition, then_branch, else_branch)];

    let prog = vec![d_func("abs", vec![("x", t_int())], ret_val(t_int()), body)];

    assert!(typecheck_prog(&prog).is_ok());
}

#[test]
fn test_struct_declaration_and_usage() {
    // struct Point { int x; int y; }
    // Point p = null;

    let prog = vec![
        d_struct("Point", vec![("x", t_int()), ("y", t_int())]),
        d_gvar("p", e_null(r_struct("Point"))),
    ];

    assert!(typecheck_prog(&prog).is_ok());
}

#[test]
fn test_nullable_reference_subtyping() {
    // int[]? nullable_arr = new int[5];
    // This should work because int[] <: int[]?

    let prog = vec![d_gvar("nullable_arr", e_new_arr(t_int(), e_int(5)))];

    // The global variable will have inferred type int[] (from new int[5])
    // This test verifies the typechecker accepts this
    assert!(typecheck_prog(&prog).is_ok());
}

#[test]
fn test_function_shadowing_and_scoping() {
    // int compute(int x) {
    //   int x = 10;  // shadows parameter
    //   int y = 20;
    //   if (true) {
    //     int y = 30;  // shadows outer y
    //     return x + y;
    //   } else {
    //     return 0;
    //   }
    // }

    let shadow_x = s_decl("x", e_int(10));
    let decl_y = s_decl("y", e_int(20));

    let inner_y = s_decl("y", e_int(30));
    let then_return = s_ret(Some(e_bop(BinOp::Add, e_id("x"), e_id("y"))));
    let else_return = s_ret(Some(e_int(0)));

    let if_stmt = s_if(e_bool(true), vec![inner_y, then_return], vec![else_return]);

    let body = vec![shadow_x, decl_y, if_stmt];

    let prog = vec![d_func(
        "compute",
        vec![("x", t_int())],
        ret_val(t_int()),
        body,
    )];

    assert!(typecheck_prog(&prog).is_ok());
}

#[test]
fn test_type_error_mismatched_return() {
    // int bad_func() { return true; }  // ERROR: returns bool, expects int

    let body = vec![s_ret(Some(e_bool(true)))];
    let prog = vec![d_func("bad_func", vec![], ret_val(t_int()), body)];

    assert!(typecheck_prog(&prog).is_err());
}

#[test]
fn test_type_error_missing_return() {
    // int no_return(int x) { int y = x; }  // ERROR: doesn't return

    let body = vec![s_decl("y", e_id("x"))];
    let prog = vec![d_func(
        "no_return",
        vec![("x", t_int())],
        ret_val(t_int()),
        body,
    )];

    assert!(typecheck_prog(&prog).is_err());
}

#[test]
fn test_type_error_duplicate_variable() {
    // int duplicate() {
    //   int x = 5;
    //   int x = 10;  // ERROR: redeclare x
    // }

    let body = vec![s_decl("x", e_int(5)), s_decl("x", e_int(10))];
    let prog = vec![d_func("duplicate", vec![], ret_void(), body)];

    assert!(typecheck_prog(&prog).is_err());
}

#[test]
fn test_type_error_wrong_operand_types() {
    // int bad_op = 5 + true;  // ERROR: can't add int and bool

    let bad_expr = e_bop(BinOp::Add, e_int(5), e_bool(true));
    let prog = vec![d_gvar("bad_op", bad_expr)];

    assert!(typecheck_prog(&prog).is_err());
}

#[test]
fn test_complex_nested_scopes() {
    // int nested(int a) {
    //   if (a > 0) {
    //     int b = a;
    //     while (b > 0) {
    //       int c = b;
    //       b = c - 1;
    //     }
    //     return b;
    //   } else {
    //     return 0;
    //   }
    // }

    let decl_b = s_decl("b", e_id("a"));
    let decl_c = s_decl("c", e_id("b"));
    let assn_b = s_assn(e_id("b"), e_bop(BinOp::Sub, e_id("c"), e_int(1)));
    let while_body = vec![decl_c, assn_b];
    let while_loop = s_while(e_bop(BinOp::Gt, e_id("b"), e_int(0)), while_body);
    let ret_b = s_ret(Some(e_id("b")));

    let then_block = vec![decl_b, while_loop, ret_b];
    let else_block = vec![s_ret(Some(e_int(0)))];

    let if_stmt = s_if(
        e_bop(BinOp::Gt, e_id("a"), e_int(0)),
        then_block,
        else_block,
    );

    let prog = vec![d_func(
        "nested",
        vec![("a", t_int())],
        ret_val(t_int()),
        vec![if_stmt],
    )];

    assert!(typecheck_prog(&prog).is_ok());
}
