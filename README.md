# Fast Expression Evaluators

[![Crates.io](https://img.shields.io/crates/v/fee.svg)](https://crates.io/crates/fee)
[![Docs.rs](https://docs.rs/fee/badge.svg)](https://docs.rs/fee)


Fast and flexible algebraic expression evaluator.

`fee` aims to evaluate strings as fast as possible once they are 
parsed, with parsing speed also being one of our concerns.The 
project was born out of the need for scientific software 
that must evaluate expressions which cannot be hardcoded in the 
source, either because the source code is not available to 
recompile, or because you want to ship a closed/private tool.

This is an early prototype (v0.1.0). While heavy optimizations
are still pending, it already achieves impressive performance.

Contributions, ideas, bug reports and recommendations are welcome.

## Usage

First step is add the dependency to your Cargo.toml.

```toml
[dependencies]
fee = { version = "0.1.1" }
```

The following code shows the default use case

```rust
use fee::{prelude::*, DefaultResolver, RpnEvaluator};

fn main()
{
    // Resolver for the variables present in the expression
    let mut var_resolver = DefaultResolver::new_empty();
    var_resolver.insert("p0".to_string(), 10.0);
    var_resolver.insert("p1".to_string(), 4.0);

    // Resolver for the functions
    let mut fn_resolver = DefaultResolver::new_empty();
    fn_resolver.insert("abs".to_string(), abs as ExprFn);

    // Shareable Context
    let context = Context::new(var_resolver, fn_resolver);

    let expr = "abs((2 + 4) * 6 / (p1 + 2)) + abs(-2)";

    // Evaluator able to parse and evaluate the given expression
    let evaluator = RpnEvaluator::new(expr).unwrap();
    let result = evaluator.eval(&context).unwrap();

    assert_eq!(result, 8.0);
}

fn abs(x: &[f64]) -> f64 {
   x[0].abs()
}
```

## Resolvers

Right now, while there is only one evaluator (RpnEvaluator),
there are five different struct that can act as resolvers for
names. Each one with is own pros and cons, so choose wisely:

- `DefaultResolver` — HashMap-based, flexible, unrestricted names, but slower.
- `IndexedResolver` — O(1), fastest for scientific/ODE-like use, restricted naming
({letter}{integer}. Ex.: p0. y100 )
- `SmallResolver` — Vec based, optimized for a few values with unrestricted names.
- `ConstantResolver` — resolves to cthe same value no matter the name.
- `EmptyResolver` — placeholder when no variables or functions are needed.

To learn more about their pros and cons read each struct's documentation.

## Features

Right now, there is only one evaluator (RpnEvaluator). It supports

- Binary operators: +, \*, -, /, ^
- Unitary operators: -
- Variables
- Functions with no limit in the number of arguments.
- f64 operations.

Example of a valid expression:

```
2 * 4 - max(abs(p0 + (-p3) * 25), p3 * 45, 0) + sqrt(5)
```

## Benches

To execute the library benches use the following script, where `$CORES` is the
CPU core range to be used. It is recommended to use the best available cores
(isolated cores if posible), with turbo disabled and a fixed frequency to reduce
noise as much as possible.

```bash
CORES=0-12
taskset -c $CORES cargo bench --features bench-internal internal
```

The following script executes the benches related to comparations with
other similar libraries available in crates.io.

```bash
CORES=0-12
taskset -c $CORES cargo bench --features bench-internal cmp
```

Right now, comparissions are being made against `evalexpr`, `meval`, `mexe` and `fasteval`.
It is planned to generate comparison plots to easily to visialize the speed difference,
as well as a table with pros and cons of each crate.
