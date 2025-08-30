use std::{sync::Arc, thread};

use fee::{prelude::*, *};

#[test]
fn test_rpn_eval_with_indexed_var_resolver()
{
    let mut var_resolver = IndexedVarResolver::new();
    var_resolver.add_identifier('p', 20);
    var_resolver.set('p', 19, 4.0);

    let fn_resolver = DefaultFnResolver::new();

    let mut context = DefaultContext::new(var_resolver, fn_resolver);

    let expr = "(2 + 4) * 6 / (p19 + 2)";
    let evaluator = RPNEvaluator::new(expr).unwrap();
    let result = evaluator.eval(&context).unwrap();
    assert_eq!(result, 6.0);

    let expr = "2 - (4 + (p19 - 2) * (p19 + 2))";
    let evaluator = RPNEvaluator::new(expr).unwrap();
    let result = evaluator.eval(&context).unwrap();
    assert_eq!(result, -14.0);

    context.vars_mut().set('p', 19, 0.0);

    let result = evaluator.eval(&context).unwrap();
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

    let context = DefaultContext::new(var_resolver, fn_resolver);

    let expr = "abs((2 + 4) * 6 / (p1 + 2)) + abs(-2)";
    let evaluator = RPNEvaluator::new(expr).unwrap();
    let result = evaluator.eval(&context).unwrap();
    assert_eq!(result, 8.0);

    let expr = "abs((2 + 4) * 6 / (p1 + 2))";
    let evaluator = RPNEvaluator::new(expr).unwrap();
    let result = evaluator.eval(&context).unwrap();
    assert_eq!(result, 6.0);

    let expr = "abs((2 * 21) + 3 - 35 - ((5 * 80) + 5) + p0)";
    let evaluator = RPNEvaluator::new(expr).unwrap();
    let result = evaluator.eval(&context).unwrap();
    assert_eq!(result, 385.0);
}

#[test]
fn test_rpn_eval_multi_threaded()
{
    let expr = "abs((2 * 21) + 3 - 35 - ((5 * 80) + 5) + p0)";

    let mut var_resolver = DefaultVarResolver::new();
    var_resolver.add_var("p0".to_string(), 10.0);
    var_resolver.add_var("p1".to_string(), 4.0);

    let mut fn_resolver = DefaultFnResolver::new();
    fn_resolver.add_fn("abs".to_string(), |args| {
        let x = args[0];
        x.abs()
    });

    let evaluator = Arc::new(RPNEvaluator::new(expr).unwrap());
    let context = Arc::new(DefaultContext::new(var_resolver, fn_resolver));

    let mut handles = vec![];

    for _ in 0..100 {
        let evaluator_clone = evaluator.clone();
        let context_clone = context.clone();

        let handle = thread::spawn(move || {
            let result = evaluator_clone.eval(context_clone.as_ref()).unwrap();
            assert_eq!(result, 385.0);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
