use fee::{*, prelude::*};

#[test]
fn test_rpn_eval_with_indexed_var_resolver()
{
    let mut var_resolver = IndexedVarResolver::new();
    var_resolver.add_identifier('p', 20);
    var_resolver.set('p', 19, 4.0);

    let fn_resolver = DefaultFnResolver::new();

    let context = Context::new(var_resolver, fn_resolver);

    let expr = "(2 + 4) * 6 / (p19 + 2)";
    let evaluator = RPNEvaluator::new(expr, &context).unwrap();
    let result = evaluator.eval();
    assert_eq!(result, 6.0);

    let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
    let evaluator = RPNEvaluator::new(expr, &context).unwrap();
    let result = evaluator.eval();
    assert_eq!(result, -14.0);
}

#[test]
fn test_rpn_eval_with_vars_and_fn()
{
    let mut var_resolver = DefaultVarResolver::new();
    var_resolver.add_var("p1".to_string(), 4.0);

    let mut fn_resolver = DefaultFnResolver::new();
    fn_resolver.add_fn("abs".to_string(), |args| {
        let x = args[0];
        x.abs()
    });

    let context = Context::new(var_resolver, fn_resolver);

    let expr = "abs((2 + 4) * 6 / (p1 + 2)) + abs(-2)";
    let evaluator = RPNEvaluator::new(expr, &context).unwrap();
    let result = evaluator.eval();
    assert_eq!(result, 8.0);
    
    let expr = "(2 + 4) * 6 / (p1 + 2)";
    let evaluator = RPNEvaluator::new(expr, &context).unwrap();
    let result = evaluator.eval();
    assert_eq!(result, 6.0);
}
