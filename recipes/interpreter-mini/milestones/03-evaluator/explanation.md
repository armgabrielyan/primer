# Explanation: 03-evaluator

Evaluation is where the earlier structure choices pay off. Because the parser already turned expressions into a tree, the evaluator can walk that tree and compute values without caring about the original character sequence.

This stage is a good example of why separating compiler or interpreter phases is useful:

- tokenizer errors are lexical
- parser errors are structural
- evaluator behavior is semantic

When those concerns are separated, bugs are easier to locate and fix.

Recursive evaluation is a natural fit here. A binary expression needs the value of its left child and the value of its right child before it can apply an operator. That makes the interpreter logic read like the tree itself.

Once this passes, the language can compute expression results end to end.
