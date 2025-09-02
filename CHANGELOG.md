# Changelog

All the changes in this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- `Context::empty()` method that returns a context with `EmptyResolver`s
- `RpnEvaluator::eval_with_stack()` method for evaluating expressions with 
a caller-provided stack.
### Fixed
### Changed
- `RpnEvaluator` no longer stores its own stack. The caller can now manage
and reuse allocated memory if desired.
### Removed
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
    - Unary negation (-)
- Created five Resolvers, each one with different pros and cons:
    - DefaultVarResolver
    - IndexedVarResolver
    - SmallVarResolver
    - ConstantResolver
    - EmptyResolver
- Feature flag `bench-internal` for running internal benchmarks.
