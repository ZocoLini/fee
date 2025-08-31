use fee::{
    EmptyResolver, RpnEvaluator, SmallResolver,
    prelude::{Context, Evaluator},
};

#[test]
fn test_lockeable_resolvers()
{
    let expr = "p0";

    let mut var_resolver = SmallResolver::new();
    var_resolver.insert("p0".to_string(), 10.0);
    let mut var_resolver = var_resolver.lock();

    let p0_ref = var_resolver.get_var_pointer("p0").unwrap();

    let fn_resolver = EmptyResolver::new();

    let context = Context::new(var_resolver, fn_resolver);
    let evaluator = RpnEvaluator::new(expr).unwrap();

    let result = evaluator.eval(&context);
    assert_eq!(result, Ok(10.0));

    unsafe {
        *p0_ref = 20.0;
    }

    let result = evaluator.eval(&context);
    assert_eq!(result, Ok(20.0));

    unsafe {
        *p0_ref = 30.0;
    }

    let result = evaluator.eval(&context);
    assert_eq!(result, Ok(30.0));
}
