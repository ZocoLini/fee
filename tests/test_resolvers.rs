use fee::{
    EmptyResolver, RpnEvaluator, SmallResolver,
    prelude::{Context},
};

#[test]
fn test_lockeable_resolvers()
{
    let expr = "p0";

    let mut var_resolver_1 = SmallResolver::new();
    var_resolver_1.insert("p0".to_string(), 10.0);
    let mut var_resolver_1 = var_resolver_1.lock();

    let mut var_resolver_2 = SmallResolver::new();
    var_resolver_2.insert("p0".to_string(), 10.0);
    let mut var_resolver_2 = var_resolver_2.lock();

    let p0_ref_1 = var_resolver_1.get_var_pointer("p0").unwrap();
    let p0_ref_2 = var_resolver_2.get_var_pointer("p0").unwrap();

    let fn_resolver_1 = EmptyResolver::new();
    let fn_resolver_2 = EmptyResolver::new();

    let context_1 = Context::new(var_resolver_1, fn_resolver_1);
    let context_2 = Context::new(var_resolver_2, fn_resolver_2);

    let evaluator = RpnEvaluator::new(expr).unwrap();

    assert_eq!(evaluator.eval(&context_1), evaluator.eval(&context_2));

    unsafe {
        *p0_ref_1 = 20.0;
        *p0_ref_2 = 20.0;
    }

    assert_eq!(evaluator.eval(&context_1), evaluator.eval(&context_2));

    unsafe {
        *p0_ref_1 = 30.0;
        *p0_ref_2 = 40.0;
    }

    assert_ne!(evaluator.eval(&context_1), evaluator.eval(&context_2));
}
