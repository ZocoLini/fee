use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use evalexpr::{DefaultNumericTypes, EvalexprError, HashMapContext, Value, context_map};
use fasteval::{Compiler, Evaler};
use fee::{IndexedResolver, UContext, prelude::*};

static SIMPLE_EXPR: &str = "3 * 3 - 3 / 3";
static POWER_EXPR: &str = "2 ^ 3 ^ 4";
static VAR_EXPR: &str = "x0 * 2";
static TRIG_EXPR: &str = "s0(x0) + c0(x0)";
static QUADRATIC_EXPR: &str = "(-x2 + (x2^2 - 4*x0*x1)^0.5) / (2*x0)";
static LARGE_EXPR: &str = "((((87))) - 73) + (97 + (((15 / 55 * ((31)) + 35))) + (15 - (9)) - (39 / 26) / 20 / 91 + 27 / (33 * 26 + 28 - (7) / 10 + 66 * 6) + 60 / 35 - ((29) - (69) / 44 / (92)) / (89) + 2 + 87 / 47 * ((2)) * 83 / 98 * 42 / (((67)) * ((97))) / (34 / 89 + 77) - 29 + 70 * (20)) + ((((((92))) + 23 * (98) / (95) + (((99) * (41))) + (5 + 41) + 10) - (36) / (6 + 80 * 52 + (90))))";

fn fee_context() -> UContext<
    IndexedResolver<Unlocked, f64>,
    IndexedResolver<Unlocked, ExprFn>,
    IndexedResolver<Locked, f64>,
    IndexedResolver<Locked, ExprFn>,
>
{
    let mut v = IndexedResolver::new();
    v.add_id('x', 3);
    v.set('x', 0, 1.0);
    v.set('x', 1, 2.0);
    v.set('x', 2, 3.0);

    let mut f = IndexedResolver::new();
    f.add_id('s', 1);
    f.set('s', 0, ExprFn::new(|args| args[0].sin())); // sin
    f.add_id('c', 1);
    f.set('c', 0, ExprFn::new(|args| args[0].cos())); // cos

    Context::new(v, f)
}

fn evalexpr_context() -> HashMapContext<DefaultNumericTypes>
{
    let context: HashMapContext<DefaultNumericTypes> = context_map! {
        "x0" => float 1,
        "x1" => float 2,
        "x2" => float 3,
        "s0" => Function::new(|argument| {
            if let Ok(float) = argument.as_float() {
                Ok(Value::Float((float as f64).sin()))
            } else {
                Err(EvalexprError::expected_number(argument.clone()))
            }
        }),
        "c0" => Function::new(|argument| {
            if let Ok(float) = argument.as_float() {
                Ok(Value::Float((float as f64).cos()))
            } else {
                Err(EvalexprError::expected_number(argument.clone()))
            }
        })
    }
    .unwrap();

    context
}

fn meval_context() -> meval::Context<'static>
{
    let mut ctx = meval::Context::empty();
    ctx.var("x0", 1.0);
    ctx.var("x1", 2.0);
    ctx.var("x2", 3.0);

    ctx.func("s0", |x| x.sin());
    ctx.func("c0", |x| x.cos());

    ctx
}

fn fasteval_namespace() -> fasteval::StrToCallbackNamespace<'static>
{
    let mut ns = fasteval::StrToCallbackNamespace::new();
    ns.insert("s0", Box::new(|x| x[0].sin()));
    ns.insert("c0", Box::new(|x| x[0].cos()));

    ns.insert("x0", Box::new(|_| 1.0));
    ns.insert("x1", Box::new(|_| 2.0));
    ns.insert("x2", Box::new(|_| 3.0));

    ns
}

fn bench_parse(c: &mut Criterion, name: &str, expr: &str)
{
    // evalexpr
    c.bench_function(&format!("cmp/parse/evalexpr/{}", name), |b| {
        use evalexpr::*;
        b.iter(|| black_box(build_operator_tree::<DefaultNumericTypes>(expr).unwrap()));
    });

    // meval
    c.bench_function(&format!("cmp/parse/meval/{}", name), |b| {
        b.iter(|| black_box(expr.parse::<meval::Expr>().unwrap()));
    });

    // fasteval
    c.bench_function(&format!("cmp/parse/fasteval/{}", name), |b| {
        let parser = fasteval::Parser::new();
        let mut slab = fasteval::Slab::new();
        b.iter(|| {
            let expr_ref = parser.parse(expr, &mut slab.ps).unwrap().from(&slab.ps);
            let _ = expr_ref.compile(&slab.ps, &mut slab.cs);
        });
    });

    // fee
    c.bench_function(&format!("cmp/parse/fee/{}", name), |b| {
        let ctx = fee_context();
        b.iter(|| black_box(Expr::compile(expr, &ctx).unwrap()));
    });
}

// ----------------- EVALUATION -----------------
fn bench_eval(c: &mut Criterion, name: &str, expr: &str)
{
    // evalexpr
    c.bench_function(&format!("cmp/eval/evalexpr/{}", name), |b| {
        use evalexpr::*;
        let precompiled = build_operator_tree::<DefaultNumericTypes>(expr).unwrap();
        let ctx = evalexpr_context();
        b.iter(|| black_box(precompiled.eval_with_context(&ctx).unwrap()));
    });

    // meval
    c.bench_function(&format!("cmp/eval/meval/{}", name), |b| {
        let parsed: meval::Expr = expr.parse().unwrap();
        let ctx = meval_context();
        b.iter(|| black_box(parsed.eval_with_context(&ctx).unwrap()));
    });

    // fasteval
    c.bench_function(&format!("cmp/eval/fasteval/{}", name), |b| {
        let mut slab = fasteval::Slab::new();
        let parser = fasteval::Parser::new();
        let expr_ref = parser.parse(expr, &mut slab.ps).unwrap().from(&slab.ps);
        let compiled = expr_ref.compile(&slab.ps, &mut slab.cs);
        let mut ns = fasteval_namespace();
        b.iter(|| black_box(compiled.eval(&slab, &mut ns).unwrap()));
    });

    // fee
    c.bench_function(&format!("cmp/eval/fee/{}", name), |b| {
        let ctx = fee_context();
        let mut stack = Vec::with_capacity(expr.len() / 2);
        let compiled = Expr::compile(expr, &ctx).unwrap();
        b.iter(|| black_box(compiled.eval(&ctx, &mut stack).unwrap()));
    });
}

fn parse_group(c: &mut Criterion)
{
    bench_parse(c, "simple", SIMPLE_EXPR);
    bench_parse(c, "power", POWER_EXPR);
    bench_parse(c, "var", VAR_EXPR);
    bench_parse(c, "trig", TRIG_EXPR);
    bench_parse(c, "quadratic", QUADRATIC_EXPR);
    bench_parse(c, "large", LARGE_EXPR);
}

fn eval_group(c: &mut Criterion)
{
    bench_eval(c, "simple", SIMPLE_EXPR);
    bench_eval(c, "power", POWER_EXPR);
    bench_eval(c, "var", VAR_EXPR);
    bench_eval(c, "trig", TRIG_EXPR);
    bench_eval(c, "quadratic", QUADRATIC_EXPR);
    bench_eval(c, "large", LARGE_EXPR);
}

criterion_group!(benches, parse_group, eval_group);
criterion_main!(benches);
