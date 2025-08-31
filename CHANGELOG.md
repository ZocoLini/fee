# Changelog

All the changes in this project are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

- Created RpnEvaluator able to evaluate mathematical expressions.
- Support for variables and functions.
- Support for the following operators:
  - Addition (+)
  - Subtraction (-)
  - Multiplication (*)
  - Division (/)
  - Exponentiation (^)
- Created three VarResolvers, each one with different pros and cons:
  - DefaultVarResolver
  - IndexedVarResolver
  - SmallVarResolver
- Created one FnResolver with support for functions with 1..n arguments.
- Added feature `bench-internal` for internal benchmarks.