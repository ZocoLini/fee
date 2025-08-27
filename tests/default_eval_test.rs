use fee::prelude::*;

#[test]
fn default_eval_test_1()
{
    let var_resolver = DefaultVarResolver::new();
    let fn_resolver = DefaultFnResolver::new();
    
    let context = Context::new(
        var_resolver,
        fn_resolver,
    );
    
    let expr = "(2 + 4) * 6";
    
    let evaluator = RPNEvaluator::new(expr, &context).unwrap();
    
    let result = evaluator.eval();
    
    assert_eq!(result, 36.0);
}

#[test]
fn default_eval_test_2()
{
    let mut var_resolver = DefaultVarResolver::new();
    
    var_resolver.add_function("p1".to_string(), 4.0);
    
    let fn_resolver = DefaultFnResolver::new();
    
    let context = Context::new(
        var_resolver,
        fn_resolver,
    );
    
    let expr = "(2 + 4) * 6 / (p1 + 2)";
    
    let evaluator = RPNEvaluator::new(expr, &context).unwrap();
    
    let result = evaluator.eval();
    
    assert_eq!(result, 6.0);
}