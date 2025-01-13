# CST (Concrete Syntax Tree)

In our programming language, the Concrete Syntax Tree (CST) plays a vital role in the compilation process. Think of the CST as a detailed map of your code, showing exactly how you wrote it, down to the smallest detail like parentheses, commas and even whitespace in some cases. It's like taking a snapshot of your code in its most raw and unprocessed form.

So, why do we need a CST? The CST is the foundation upon which we build deeper understanding of your code. Pest gets us a long way on our AST journey but not close enough to an actually useful AST so we use it to get this far and then we move out.

We also use the CST as the input to the language server to make refactoring, formatting and debugging easier and more accurate.

Once we have the CST, we convert it into an Abstract Syntax Tree (AST). The AST strips away unnecessary details (like extra whitespace or redundant parentheses) and focuses on the structure and meaning of your program. The AST adds more information, such as:

* References and borrows: Who owns what, and how things are being accessed.
* Basic type information: Understanding what types your variables and expressions are using.
* Program structure: How everything fits together logically.


This richer, more abstract representation is what gets sent to the type engine and other stages of the compiler for further analysis and transformation.

In short, the CST is like the detailed blueprint of your code, and the AST is the cleaned-up, annotated version that lets us reason about your program effectively. Together, they ensure your code is processed accurately and efficiently, step by step.
