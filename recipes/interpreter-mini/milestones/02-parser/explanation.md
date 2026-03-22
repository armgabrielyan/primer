# Explanation: 02-parser

The parser turns a flat token stream into structure. In arithmetic languages, that structure has to preserve precedence, because `1 + 2 * 3` and `(1 + 2) * 3` mean different things even though they use the same symbols.

An AST is useful because it separates "what the program means" from "what the source text looked like." Later stages can evaluate the tree or transform it without re-reading raw tokens.

Recursive descent is a common beginner-friendly parsing strategy because each function can represent one grammar level. You can read the code almost like a grammar:

- expression
- term
- factor
- primary

Once this milestone passes, you have a structured representation that the evaluator can walk directly.
