# Changelog

All the changes in this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- `Context::empty()` method that returns a context with `EmptyResolver`s
- `RpnExpr`and `IRpnExpr` structs exposed to the library user.
### Fixed
- Exponentiation had precedence over the unary minus operator.
- Fixed parsing nested functions with multiple arguments.
### Changed
- The `Expr` structs are now the ones that expose the specialized `eval()` method.
- `DefaultVarResolver` and `SmallVarResolver` allows keys different to String.
### Removed
- Removed the `Evaluator` concept and the `RpnEvaluator` struct.
### Security
### Deprecated

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
