Language Structure via Graphs

Directed and undirected graphs for different aspects: AST as a directed graph for program structure, undirected graphs for memory tracking and type information.
Graph structure allows efficient traversal, insertion, and updates, suitable for various language functions.
Sub-Graphs and Data Localization:

Preference for using sub-graphs rather than separate graphs to keep data localized within specific scopes or contexts.
Data localization through sub-graphs would support modularity and control over the program’s functional domains, particularly for types, memory, and ownership.
Existing Language Inspirations:

Rust: Borrow checker as a form of constraint graph for managing memory safety.
Haskell: Type system with a graph-like hierarchy based on category theory.
Datalog and Soufflé: Directed graph approach for logical inference and dependency management.
LLVM IR: Represents program data flow as a DAG (directed acyclic graph), supporting SSA form and dependencies.
Eff and Links Language Insights:

Graphs for effect tracking and dependency management in Eff and Links surface data dependency issues and functional impurities.
Dependency graphs in these languages are used to maintain effect safety, purity, and localized control over side effects.
Effect sub-typing and region-based effect systems organize effects into sub-graphs or regions, enforcing functional boundaries and preventing unintended state propagation.
