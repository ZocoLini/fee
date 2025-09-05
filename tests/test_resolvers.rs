use fee::{EmptyResolver, LRpnExpr, SmallResolver, prelude::*};

#[test]
fn test_lockeable_resolvers()
{
    let expr = "p0";

    let mut var_resolver_1 = SmallResolver::new();
    var_resolver_1.insert("p0".to_string(), 10.0);
    let var_resolver_1 = var_resolver_1.lock();

    let mut var_resolver_2 = SmallResolver::new();
    var_resolver_2.insert("p0".to_string(), 10.0);
    let var_resolver_2 = var_resolver_2.lock();

    let fn_resolver_1 = EmptyResolver::new();
    let fn_resolver_2 = EmptyResolver::new();

    let context_1 = Context::new(var_resolver_1, fn_resolver_1);
    let context_2 = Context::new(var_resolver_2, fn_resolver_2);

    let p0_ptr_1 = context_1.vars().get_ptr("p0").unwrap();
    let p0_ptr_2 = context_2.vars().get_ptr("p0").unwrap();

    let rpn_expr_1 = Expr::compile_locked(expr, &context_1).unwrap();
    let rpn_expr_2 = Expr::compile_locked(expr, &context_2).unwrap();

    let mut stack = Vec::with_capacity(rpn_expr_1.len() / 2);

    assert_eq!(
        rpn_expr_1.eval_locked(&context_1, &mut stack),
        rpn_expr_2.eval_locked(&context_2, &mut stack)
    );

    p0_ptr_1.set(20.0);
    p0_ptr_2.set(20.0);

    assert_eq!(
        rpn_expr_1.eval_locked(&context_1, &mut stack),
        rpn_expr_2.eval_locked(&context_2, &mut stack)
    );

    p0_ptr_1.set(50.0);
    p0_ptr_2.set(40.0);

    assert_ne!(
        rpn_expr_1.eval_locked(&context_1, &mut stack),
        rpn_expr_2.eval_locked(&context_2, &mut stack)
    );
}
