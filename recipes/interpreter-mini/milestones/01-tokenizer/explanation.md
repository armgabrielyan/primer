# Explanation: 01-tokenizer

Tokenization is the stage where raw source text becomes a sequence of meaningful symbols. Instead of reasoning about individual characters everywhere in the interpreter, later stages can work with higher-level pieces such as numbers, identifiers, and operators.

This separation matters because each stage has a different job:

- the tokenizer recognizes lexical units
- the parser recognizes structure
- the evaluator computes meaning

Even in a tiny language, keeping those boundaries clear makes the code easier to debug. If the wrong token types come out of the scanner, the parser does not need to be blamed yet. You can inspect the token stream directly and narrow the problem down quickly.

Once this milestone passes, you have a stable representation that later milestones can trust.
