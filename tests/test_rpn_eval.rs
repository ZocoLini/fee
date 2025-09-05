use std::{sync::Arc, thread};

use fee::{prelude::*, *};

#[test]
fn test_rpn_eval_with_indexed_var_resolver()
{
    let mut var_resolver = IndexedResolver::new_var_resolver();
    var_resolver.add_var_identifier('p', 20);
    var_resolver.set('p', 19, 4.0);

    let fn_resolver = EmptyResolver::new();

    let mut context = Context::new_unlocked(var_resolver, fn_resolver);

    let expr = "(2 + 4) * 6 / (p19 + 2)";
    let expr = Expr::compile(expr, &context).unwrap();
    let result = expr.eval(&context, &mut Vec::new()).unwrap();
    assert_eq!(result, 6.0);

    let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
    let expr = Expr::compile(expr, &context).unwrap();
    let result = expr.eval(&context, &mut Vec::new()).unwrap();
    assert_eq!(result, -14.0);

    context.vars_mut().set('p', 19, 0.0);

    let result = expr.eval(&context, &mut Vec::new()).unwrap();
    assert_eq!(result, 2.0);
}

#[test]
fn test_rpn_eval_with_vars_and_fn()
{
    let mut var_resolver = DefaultResolver::new_var_resolver();
    var_resolver.insert("p0".to_string(), 10.0);
    var_resolver.insert("p1".to_string(), 4.0);

    let mut fn_resolver = DefaultResolver::new_fn_resolver();
    fn_resolver.insert("abs".to_string(), |args| {
        let x = args[0];
        x.abs()
    });

    let context = Context::new_unlocked(var_resolver, fn_resolver);
    let mut stack = Vec::with_capacity(10);

    let expr = "-abs((2 + 4) * 6 / (p1 + 2)) + abs(-2)";
    let expr = Expr::compile(expr, &context).unwrap();
    let result = expr.eval(&context, &mut stack).unwrap();
    assert_eq!(result, -4.0);

    let expr = "abs((2 + 4) * 6 / (p1 + 2))";
    let expr = Expr::compile(expr, &context).unwrap();
    let result = expr.eval(&context, &mut stack).unwrap();
    assert_eq!(result, 6.0);

    let expr = "abs((2 * 21) + 3 - 35 + (-((5 * 80) + 5)) + p0)";
    let expr = Expr::compile(expr, &context).unwrap();
    let result = expr.eval(&context, &mut stack).unwrap();
    assert_eq!(result, 385.0);

    let expr = "-3^2 + (-3)^2";
    let expr = Expr::compile(expr, &context).unwrap();
    let result = expr.eval(&context, &mut stack).unwrap();
    assert_eq!(result, 0.0);
}

#[test]
fn test_rpn_eval_multi_threaded()
{
    let expr = "abs((2 * 21) + 3 - 35 - ((5 * 80) + 5) + p0)";

    let mut var_resolver = DefaultResolver::new_var_resolver();
    var_resolver.insert("p0".to_string(), 10.0);
    var_resolver.insert("p1".to_string(), 4.0);

    let mut fn_resolver = DefaultResolver::new_fn_resolver();
    fn_resolver.insert("abs".to_string(), |args| {
        let x = args[0];
        x.abs()
    });

    let context = Arc::new(Context::new_unlocked(var_resolver, fn_resolver));
    let expr = Arc::new(Expr::compile(expr, &context).unwrap());

    let mut handles = vec![];

    for _ in 0..100 {
        let context_clone = context.clone();
        let expr_clone = expr.clone();

        let mut stack = Vec::new();

        let handle = thread::spawn(move || {
            let result = expr_clone.eval(context_clone.as_ref(), &mut stack).unwrap();
            assert_eq!(result, 385.0);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
