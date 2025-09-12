use std::f64;

use fee::{prelude::*, *};

fn f0(args: &[f64]) -> f64
{
    args[0].sqrt()
}

fn f1(args: &[f64]) -> f64
{
    args[0].abs()
}

fn f2(args: &[f64]) -> f64
{
    if args[0] > args[1] { args[0] } else { args[1] }
}

fn f3(args: &[f64]) -> f64
{
    args[0] as i64 as f64
}

#[test]
fn test_eval_pipelines()
{
    let expr = r#"
f3(-((p0^2 + (3 * p1 - (p2^3))) - (-(p3^2) + f0((p4 - p5)^2 + (p6^2))))
+ f1(((p7^3) - (-(p8^2))))
+ f2((p9 - (p10 - (p11^2))), 3))"#;

    let mut stack = Vec::with_capacity(20);

    // Default RPN
    {
        let mut var_resolver = DefaultResolver::empty();
        var_resolver.insert("p0", 0.0);
        var_resolver.insert("p1", 1.0);
        var_resolver.insert("p2", 2.0);
        var_resolver.insert("p3", 3.0);
        var_resolver.insert("p4", 4.0);
        var_resolver.insert("p5", 5.0);
        var_resolver.insert("p6", 6.0);
        var_resolver.insert("p7", 7.0);
        var_resolver.insert("p8", 8.0);
        var_resolver.insert("p9", 9.0);
        var_resolver.insert("p10", 10.0);
        var_resolver.insert("p11", 11.0);

        let mut fn_resolver = DefaultResolver::empty();
        fn_resolver.insert("f0", ExprFn::new(f0));
        fn_resolver.insert("f1", ExprFn::new(f1));
        fn_resolver.insert("f2", ExprFn::new(f2));
        fn_resolver.insert("f3", ExprFn::new(f3));

        let context = Context::new(var_resolver, fn_resolver);

        let expr = Expr::compile(expr, &context).unwrap();
        assert_eq!(expr.eval(&context, &mut stack), Ok(529.0));
    };

    // Indexed Vars RPN
    {
        let mut var_resolver = IndexedResolver::new();
        var_resolver.add_id('p', 12);
        var_resolver.set('p', 0, 0.0);
        var_resolver.set('p', 1, 1.0);
        var_resolver.set('p', 2, 2.0);
        var_resolver.set('p', 3, 3.0);
        var_resolver.set('p', 4, 4.0);
        var_resolver.set('p', 5, 5.0);
        var_resolver.set('p', 6, 6.0);
        var_resolver.set('p', 7, 7.0);
        var_resolver.set('p', 8, 8.0);
        var_resolver.set('p', 9, 9.0);
        var_resolver.set('p', 10, 10.0);
        var_resolver.set('p', 11, 11.0);

        let mut fn_resolver = DefaultResolver::empty();
        fn_resolver.insert("f0", ExprFn::new(f0));
        fn_resolver.insert("f1", ExprFn::new(f1));
        fn_resolver.insert("f2", ExprFn::new(f2));
        fn_resolver.insert("f3", ExprFn::new(f3));

        let context = Context::new(var_resolver, fn_resolver);

        let expr = Expr::compile(expr, &context).unwrap();
        assert_eq!(expr.eval(&context, &mut stack), Ok(529.0));
    };

    // Indexed Fns RPN
    {
        let mut var_resolver = DefaultResolver::empty();
        var_resolver.insert("p0", 0.0);
        var_resolver.insert("p1", 1.0);
        var_resolver.insert("p2", 2.0);
        var_resolver.insert("p3", 3.0);
        var_resolver.insert("p4", 4.0);
        var_resolver.insert("p5", 5.0);
        var_resolver.insert("p6", 6.0);
        var_resolver.insert("p7", 7.0);
        var_resolver.insert("p8", 8.0);
        var_resolver.insert("p9", 9.0);
        var_resolver.insert("p10", 10.0);
        var_resolver.insert("p11", 11.0);

        let mut fn_resolver = IndexedResolver::new();
        fn_resolver.add_id('f', 4);
        fn_resolver.set('f', 0, ExprFn::new(f0));
        fn_resolver.set('f', 1, ExprFn::new(f1));
        fn_resolver.set('f', 2, ExprFn::new(f2));
        fn_resolver.set('f', 3, ExprFn::new(f3));

        let context = Context::new(var_resolver, fn_resolver);

        let expr = Expr::compile(expr, &context).unwrap();
        assert_eq!(expr.eval(&context, &mut stack), Ok(529.0));
    };

    // Full Indexed RPN
    {
        let mut var_resolver = IndexedResolver::new();
        var_resolver.add_id('p', 12);
        var_resolver.set('p', 0, 0.0);
        var_resolver.set('p', 1, 1.0);
        var_resolver.set('p', 2, 2.0);
        var_resolver.set('p', 3, 3.0);
        var_resolver.set('p', 4, 4.0);
        var_resolver.set('p', 5, 5.0);
        var_resolver.set('p', 6, 6.0);
        var_resolver.set('p', 7, 7.0);
        var_resolver.set('p', 8, 8.0);
        var_resolver.set('p', 9, 9.0);
        var_resolver.set('p', 10, 10.0);
        var_resolver.set('p', 11, 11.0);

        let mut fn_resolver = IndexedResolver::new();
        fn_resolver.add_id('f', 4);
        fn_resolver.set('f', 0, ExprFn::new(f0));
        fn_resolver.set('f', 1, ExprFn::new(f1));
        fn_resolver.set('f', 2, ExprFn::new(f2));
        fn_resolver.set('f', 3, ExprFn::new(f3));

        let context = Context::new(var_resolver, fn_resolver);

        let expr = Expr::compile(expr, &context).unwrap();
        assert_eq!(expr.eval(&context, &mut stack), Ok(529.0));
    };

    // Locked RPN
    {
        let mut var_resolver = DefaultResolver::empty();
        var_resolver.insert("p0".to_string(), 0.0);
        var_resolver.insert("p1".to_string(), 1.0);
        var_resolver.insert("p2".to_string(), 2.0);
        var_resolver.insert("p3".to_string(), 3.0);
        var_resolver.insert("p4".to_string(), 4.0);
        var_resolver.insert("p5".to_string(), 5.0);
        var_resolver.insert("p6".to_string(), 6.0);
        var_resolver.insert("p7".to_string(), 7.0);
        var_resolver.insert("p8".to_string(), 8.0);
        var_resolver.insert("p9".to_string(), 9.0);
        var_resolver.insert("p10".to_string(), 10.0);
        var_resolver.insert("p11".to_string(), 11.0);

        let mut fn_resolver = DefaultResolver::empty();
        fn_resolver.insert("f0".to_string(), ExprFn::new(f0));
        fn_resolver.insert("f1".to_string(), ExprFn::new(f1));
        fn_resolver.insert("f2".to_string(), ExprFn::new(f2));
        fn_resolver.insert("f3".to_string(), ExprFn::new(f3));

        let context = Context::new(var_resolver, fn_resolver).lock();

        let expr = Expr::compile(expr, &context).unwrap();
        assert_eq!(expr.eval(&context, &mut stack), Ok(529.0));

        let p0_ptr = context.get_var_ptr("p0").unwrap();
        p0_ptr.set(1.0);

        assert_eq!(expr.eval(&context, &mut stack), Ok(528.0));
    };
}

#[test]
fn test_eval_operators()
{
    let ctx = Context::empty();
    let mut stack = Vec::new();

    let expr = "3 * 3 - (-4) / 4 + 4 % 2 + 4^3";
    let expr = Expr::compile(expr, &ctx).unwrap();
    assert_eq!(expr.eval(&ctx, &mut stack).unwrap(), 74.0);

    let expr = "!false == 1.0 && !0.0 == true";
    let expr = Expr::compile(expr, &ctx).unwrap();
    assert_eq!(expr.eval(&ctx, &mut stack).unwrap(), 1.0);

    let expr = "3 > 4 && false || 3 >= 3 && 3 <= 3 && 3 < 4 && 5 != 6";
    let expr = Expr::compile(expr, &ctx).unwrap();
    assert_eq!(expr.eval(&ctx, &mut stack).unwrap(), 1.0);

    let expr = "1 << 3 == 8 && 8 >> 3 == 1";
    let expr = Expr::compile(expr, &ctx).unwrap();
    assert_eq!(expr.eval(&ctx, &mut stack).unwrap(), 1.0);

    let expr = "6 & 3 == 2 && 6 | 3 == 7 && 6 ^^ 3 == 5";
    let expr = Expr::compile(expr, &ctx).unwrap();
    assert_eq!(expr.eval(&ctx, &mut stack).unwrap(), 1.0);
}

#[test]
fn test_hard_expressions()
{
    fn min(x: &[f64]) -> f64
    {
        let mut res = f64::MAX;

        for x in x.iter() {
            if *x < res {
                res = *x;
            }
        }

        res
    }
    fn max(x: &[f64]) -> f64
    {
        let mut res = f64::MIN;

        for x in x.iter() {
            if *x > res {
                res = *x;
            }
        }

        res
    }

    let mut v_resolver = SmallResolver::new();
    v_resolver.insert("x", 0.0);
    v_resolver.insert("y", 0.0);
    v_resolver.insert("z", 1.0);
    v_resolver.insert("a", 1.0);
    v_resolver.insert("b", 1.0);
    v_resolver.insert("c", 10.0);

    let mut f_resolver = SmallResolver::new();
    f_resolver.insert("max", ExprFn::new(max));
    f_resolver.insert("min", ExprFn::new(min));
    f_resolver.insert("abs", ExprFn::new(|x| x[0].abs()));

    let ctx = Context::new(v_resolver, f_resolver);
    let mut stack = Vec::new();

    let expr = "
        ((x + 3 * y) << 2) & 255 | ((10 ^ 2) % 7)
        ^^ (true && (z > 5 || false))
        + max(a, b, c)
        - min(1, 2, 3)
        * abs(-42)
        ";
    let expr = Expr::compile(expr, &ctx).unwrap();
    assert_eq!(expr.eval(&ctx, &mut stack).unwrap(), -30.0);
}
