use fee::{DefaultResolver, prelude::*};

#[test]
fn test_ptr()
{
    let mut var_resolver = DefaultResolver::empty();
    var_resolver.insert("p0", 10.0);

    let mut fn_resolver = DefaultResolver::empty();
    fn_resolver.insert("f0", ExprFn::new(|_| 0.0));

    let context = Context::new(var_resolver, fn_resolver).lock();

    let p0_ptr = context.get_var_ptr("p0").unwrap();
    let f0_ptr = context.get_fn_ptr("f0").unwrap();

    p0_ptr.set(20.0);
    f0_ptr.set(ExprFn::new(|_| 20.0));

    assert_eq!(p0_ptr.get(), f0_ptr.get()(&[0.0; 0]))
}
