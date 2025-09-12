# Changelog

All the changes in this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## Unreleased

## Added
- Added `stack underflow error` now is returned when an operator doesn't
have enough elements in the stack.
- Added support for `logical operators`, `comparisons`, and `bitwise operations`.
- Added `true` and `false` keywords.

## [0.2.1] - 2025-09-11

### Added
- Added support for the module operator (`%`)
### Changed
- Exponentiation (`^`) now uses powi internally in integer cases.
- Parsing into RPN no longer makes an intermediate step through infix 
notation, improving parsing performance by 40–75%.

## [0.2.0] - 2025-09-08

### Added
- `Context::empty()` method returning a context with `EmptyResolver`s
- `ExprCompiler` and `ExprEvaluator` traits for the different `Expr<T>` types.
- `Ptr` struct for handling raw pointers to resolver contents.
- New `Expr` variant optimized for locked `Context`s
- New `Expr` variants optimized for indexed `Resolver`s
### Fixed
- Parsing of nested functions with multiple arguments.
### Changed
- `Expr<T>` now directly exposes the specialized `eval()` and `compile()` methods.
- `DefaultVarResolver` and `SmallVarResolver` now allow keys other than String.
- Improved public interfaces of several resolvers.
- `Resolver`s can no longer be locked individually, locking is now managed by the owning `Context`.
- `ExprFn` is no longer a type alias, it is now a struct.
- Evaluating expressions now requires you to provide the stack.
### Removed
- Removed the `Evaluator` concept and the `RpnEvaluator` struct.

## [0.1.1] - 2025-09-03

### Fixed
- `Unary minus (-)` operator no longer incorrectly takes precedence over 
`exponentiation (^)`.

## [0.1.0] - 2025-09-02

### Added
- Created RpnEvaluator able to evaluate mathematical expressions.
- Support for variables and functions.
- Support for the following operators:
    - Addition (+)
    - Subtraction (-)
    - Multiplication (\*)
    - Division (/)
    - Exponentiation (^)
    - Unary minus (-)
- Created five Resolvers, each one with different pros and cons:
    - DefaultVarResolver
    - IndexedVarResolver
    - SmallVarResolver
    - ConstantResolver
    - EmptyResolver
- Feature flag `bench-internal` for running internal benchmarks.
