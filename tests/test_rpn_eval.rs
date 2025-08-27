use fee::{DefaultFnResolver, DefaultVarResolver, IndexedVarResolver, RPNEvaluator, prelude::*};

#[test]
fn test_rpn_eval_without_vars()
{
    let var_resolver = DefaultVarResolver::new();
    let fn_resolver = DefaultFnResolver::new();

    let context = Context::new(var_resolver, fn_resolver);

    let expr = "(2 + 4) * 6";

    let evaluator = RPNEvaluator::new(expr, &context).unwrap();

    let result = evaluator.eval();

    assert_eq!(result, 36.0);
}

#[test]
fn test_rpn_eval_with_vars()
{
    let mut var_resolver = DefaultVarResolver::new();

    var_resolver.add_var("p1".to_string(), 4.0);

    let fn_resolver = DefaultFnResolver::new();

    let context = Context::new(var_resolver, fn_resolver);

    let expr = "(2 + 4) * 6 / (p1 + 2)";

    let evaluator = RPNEvaluator::new(expr, &context).unwrap();

    let result = evaluator.eval();

    assert_eq!(result, 6.0);
}

#[test]
fn test_rpn_eval_2_with_indexed_var_resolver()
{
    let mut var_resolver = IndexedVarResolver::new();
    var_resolver.add_identifier('p', 1);

    var_resolver.set('p', 0, 4.0);

    let fn_resolver = DefaultFnResolver::new();

    let context = Context::new(var_resolver, fn_resolver);

    let expr = "(2 + 4) * 6 / (p0 + 2)";

    let evaluator = RPNEvaluator::new(expr, &context).unwrap();

    let result = evaluator.eval();

    assert_eq!(result, 6.0);
}
