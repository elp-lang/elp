pub mod syntax_kind;

pub use syntax_kind::{ElpLanguage, SyntaxKind};

pub type SyntaxNode = rowan::SyntaxNode<ElpLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<ElpLanguage>;
pub type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

/// Typed wrapper over a [`SyntaxNode`] of a specific [`SyntaxKind`] — the
/// rust-analyzer `ast::AstNode` pattern. A parser crate defines one of
/// these per grammar rule it cares about; the wrapper's accessor methods
/// read data by walking `self.syntax()`'s children/tokens on demand rather
/// than storing pre-extracted fields, so the tree is always the source of
/// truth.
pub trait AstNode {
    fn can_cast(kind: SyntaxKind) -> bool;

    fn cast(node: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;
}

/// Thin wrapper over `rowan::GreenNodeBuilder` in `SyntaxKind` terms, so
/// callers (the parser, as it walks tokens) never touch raw `rowan::SyntaxKind`.
#[derive(Default)]
pub struct SyntaxTreeBuilder {
    builder: rowan::GreenNodeBuilder<'static>,
}

impl SyntaxTreeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(to_raw(kind));
    }

    pub fn token(&mut self, kind: SyntaxKind, text: &str) {
        self.builder.token(to_raw(kind), text);
    }

    pub fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    /// Marks a point in the tree so a node can be retroactively wrapped
    /// around everything built since — the standard way a Pratt/precedence-
    /// climbing parser handles left-recursive binary expressions with rowan:
    /// parse the LHS first (without knowing yet whether it'll turn out to be
    /// the left side of a binary expression), take a checkpoint, and only
    /// once a binary operator actually shows up, retroactively open a
    /// `BINARY_EXPR` node starting at that checkpoint.
    pub fn checkpoint(&mut self) -> Checkpoint {
        Checkpoint(self.builder.checkpoint())
    }

    /// Opens a node that starts at `checkpoint` (in the past) rather than
    /// "right now" — everything built since the checkpoint becomes this
    /// node's children.
    pub fn start_node_at(&mut self, checkpoint: Checkpoint, kind: SyntaxKind) {
        self.builder.start_node_at(checkpoint.0, to_raw(kind));
    }

    /// Consumes the builder and returns the finished tree, rooted at
    /// whichever node is still open when the last `finish_node` runs.
    pub fn finish(self) -> SyntaxNode {
        SyntaxNode::new_root(self.builder.finish())
    }
}

/// Opaque handle from [`SyntaxTreeBuilder::checkpoint`]; see its docs.
/// `Copy` because the same checkpoint is routinely reused across multiple
/// loop iterations — e.g. wrapping each step of a left-associative binary
/// or postfix chain (`1 - 2 - 3`, `a.b.c`) around the same starting point.
#[derive(Clone, Copy)]
pub struct Checkpoint(rowan::Checkpoint);

fn to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
    <ElpLanguage as rowan::Language>::kind_to_raw(kind)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Hand-builds the tree an `import { test } from my.module` parse would
    // produce, to prove the builder/SyntaxKind/rowan wiring is correct end
    // to end before crates/parser is wired up to drive it for real.
    #[test]
    fn builds_a_tree_shaped_like_an_import_statement() {
        let mut builder = SyntaxTreeBuilder::new();

        builder.start_node(SyntaxKind::SCRIPT);
        builder.start_node(SyntaxKind::IMPORT_STMT);
        builder.token(SyntaxKind::IMPORT_KW, "import");
        builder.token(SyntaxKind::WHITESPACE, " ");
        builder.token(SyntaxKind::LCURLY, "{");
        builder.token(SyntaxKind::WHITESPACE, " ");

        builder.start_node(SyntaxKind::IMPORT_LIST);
        builder.start_node(SyntaxKind::IMPORT_NAME);
        builder.token(SyntaxKind::IDENT, "test");
        builder.finish_node(); // IMPORT_NAME
        builder.finish_node(); // IMPORT_LIST

        builder.token(SyntaxKind::WHITESPACE, " ");
        builder.token(SyntaxKind::RCURLY, "}");
        builder.token(SyntaxKind::WHITESPACE, " ");
        builder.token(SyntaxKind::FROM_KW, "from");
        builder.token(SyntaxKind::WHITESPACE, " ");

        builder.start_node(SyntaxKind::MODULE_PATH);
        builder.token(SyntaxKind::IDENT, "my");
        builder.token(SyntaxKind::DOT, ".");
        builder.token(SyntaxKind::IDENT, "module");
        builder.finish_node(); // MODULE_PATH

        builder.finish_node(); // IMPORT_STMT
        builder.finish_node(); // SCRIPT

        let root = builder.finish();

        assert_eq!(root.kind(), SyntaxKind::SCRIPT);
        assert_eq!(root.text(), "import { test } from my.module");

        let import_stmt = root.first_child().expect("import stmt child");
        assert_eq!(import_stmt.kind(), SyntaxKind::IMPORT_STMT);

        let import_list = import_stmt
            .children()
            .find(|n| n.kind() == SyntaxKind::IMPORT_LIST)
            .expect("import list child");
        let import_name = import_list.first_child().expect("import name child");
        assert_eq!(import_name.kind(), SyntaxKind::IMPORT_NAME);
        assert_eq!(import_name.text(), "test");

        let module_path = import_stmt
            .children()
            .find(|n| n.kind() == SyntaxKind::MODULE_PATH)
            .expect("module path child");
        assert_eq!(module_path.text(), "my.module");

        // Full-fidelity round trip: every byte of the original source,
        // including whitespace, is recoverable from the tree.
        assert_eq!(root.text().to_string(), "import { test } from my.module");
    }

    // Proves the checkpoint/start_node_at mechanism works before the parser
    // relies on it for left-recursive binary expressions (`1 + 2 * 3`):
    // the LHS is built without knowing it'll need wrapping, then wrapped
    // retroactively once a binary operator shows up.
    #[test]
    fn checkpoint_retroactively_wraps_a_binary_expression() {
        let mut builder = SyntaxTreeBuilder::new();

        let checkpoint = builder.checkpoint();
        builder.token(SyntaxKind::INTEGER, "1");
        builder.start_node_at(checkpoint, SyntaxKind::BINARY_EXPR);
        builder.token(SyntaxKind::PLUS, "+");
        builder.token(SyntaxKind::INTEGER, "2");
        builder.finish_node(); // BINARY_EXPR

        let root = builder.finish();
        assert_eq!(root.kind(), SyntaxKind::BINARY_EXPR);
        assert_eq!(root.text(), "1+2");
        assert_eq!(root.children_with_tokens().count(), 3);
    }
}
