use fee::{prelude::*, *};

#[test]
fn test_rpn_eval_with_indexed_var_resolver()
{
    let mut var_resolver = IndexedVarResolver::new();
    var_resolver.add_identifier('p', 20);
    var_resolver.set('p', 19, 4.0);

    let fn_resolver = DefaultFnResolver::new();

    let mut context = Context::new(var_resolver, fn_resolver);

    let expr = "(2 + 4) * 6 / (p19 + 2)";
    let evaluator = RPNEvaluator::new(expr, &mut context).unwrap();
    let result = evaluator.eval().unwrap();
    assert_eq!(result, 6.0);

    let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
    let mut evaluator = RPNEvaluator::new(expr, &mut context).unwrap();
    let result = evaluator.eval().unwrap();
    assert_eq!(result, -14.0);

    let context = evaluator.context_mut();
    context.vars_mut().set('p', 19, 0.0);

    let result = evaluator.eval().unwrap();
    assert_eq!(result, 2.0);
}

#[test]
fn test_rpn_eval_with_vars_and_fn()
{
    let mut var_resolver = DefaultVarResolver::new();
    var_resolver.add_var("p0".to_string(), 10.0);
    var_resolver.add_var("p1".to_string(), 4.0);

    let mut fn_resolver = DefaultFnResolver::new();
    fn_resolver.add_fn("abs".to_string(), |args| {
        let x = args[0];
        x.abs()
    });

    let mut context = Context::new(var_resolver, fn_resolver);

    let expr = "abs((2 + 4) * 6 / (p1 + 2)) + abs(-2)";
    let evaluator = RPNEvaluator::new(expr, &mut context).unwrap();
    let result = evaluator.eval().unwrap();
    assert_eq!(result, 8.0);

    let expr = "abs((2 + 4) * 6 / (p1 + 2))";
    let evaluator = RPNEvaluator::new(expr, &mut context).unwrap();
    let result = evaluator.eval().unwrap();
    assert_eq!(result, 6.0);

    let expr = "abs((2 * 21) + 3 - 35 - ((5 * 80) + 5) + p0)";
    let evaluator = RPNEvaluator::new(expr, &mut context).unwrap();
    let result = evaluator.eval().unwrap();
    assert_eq!(result, 385.0);
}
