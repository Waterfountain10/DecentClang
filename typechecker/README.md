Typechecker for Oat. Validates types and control flow before code generation.

Checks subtyping for reference types, nullable references, and function signatures. Infers types for expressions. Validates statements including assignments, declarations, returns, and control flow. Ensures non-void functions return on all paths.

Program typechecking runs in two passes. First pass registers struct declarations. Second pass processes global variables and functions, checking function bodies after signatures are added to context.

Entry point is typecheck_prog which takes an AST and returns a result. On error returns TypeError with message and span.

Usage:

```rust
    use typechecker::typecheck_prog;
    let program = /* parsed AST */;
    typecheck_prog(&program)?;
```

The typechecker does not modify the AST.
