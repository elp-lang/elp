use std::{collections::HashMap, iter::Peekable, str::Chars};

/// A simple trie for fast prefix-matching of multi-character symbols.
///
/// This structure is useful for lexers that need to distinguish between
/// overlapping or multi-character symbols, such as:
/// - `=`, `==`, `===`
/// - `->`, `=>`, `>>`
/// - etc.
///
/// The Trie` stores a set of known symbols and allows efficient
/// longest-prefix lookup to the Val value.
///
/// # Example
///
/// ```rust
/// use lexer::trie::Trie;
/// use lexer::tokens::LexerSymbol;
///
/// let mut trie = Trie::default();
/// trie.insert("==", LexerSymbol::EqualEqual);
/// trie.insert("=", LexerSymbol::Equal);
/// trie.insert("->", LexerSymbol::Arrow);
///
/// let result = trie.match_longest(&mut "===".chars().peekable());
/// assert_eq!(result, Some((LexerSymbol::EqualEqual, "==".into())));
/// ```
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Trie<Val: Default + Clone> {
    children: HashMap<char, Trie<Val>>,
    token: Option<Val>,
}

impl<Val: Default + Clone> Trie<Val> {
    pub fn insert(&mut self, s: &str, sym: Val) {
        let mut node = self;
        for c in s.chars() {
            node = node.children.entry(c).or_default();
        }
        node.token = Some(sym);
    }

    pub fn match_longest(&self, chars: &mut Peekable<Chars>) -> Option<(Val, String)> {
        let mut node = self;
        let mut len = 0;
        let mut last = None;
        let mut temp = chars.clone();
        let mut out_chars: String = "".into();

        while let Some(&c) = temp.peek() {
            if let Some(next_node) = node.children.get(&c) {
                len += c.len_utf8();
                out_chars.push(temp.next()?);
                if let Some(token) = &next_node.token {
                    last = Some((token, len));
                }
                node = next_node;
            } else {
                break;
            }
        }

        if let Some((token, len)) = last {
            for _ in 0..len {
                chars.next();
            }
            Some((token.clone(), out_chars))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tokens::LexerSymbol;

    use super::*;

    fn make_trie() -> Trie<LexerSymbol> {
        let mut trie = Trie::<LexerSymbol>::default();
        trie.insert("=", LexerSymbol::Equal);
        trie.insert("==", LexerSymbol::EqualEqual);
        trie.insert("->", LexerSymbol::Arrow);
        trie.insert("+", LexerSymbol::Plus);
        trie.insert("-", LexerSymbol::Minus);
        trie.insert("+=", LexerSymbol::PlusEqual);
        trie
    }

    fn match_symbol<Val: Default + Clone>(trie: &Trie<Val>, input: &str) -> Option<(Val, String)> {
        let mut chars = input.chars().peekable();
        trie.match_longest(&mut chars)
    }

    #[test]
    fn test_exact_match_single_char() {
        let trie = make_trie();
        assert_eq!(
            match_symbol(&trie, "="),
            Some((LexerSymbol::Equal, "=".into()))
        );
        assert_eq!(
            match_symbol(&trie, "+"),
            Some((LexerSymbol::Plus, "+".into()))
        );
        assert_eq!(
            match_symbol(&trie, "-"),
            Some((LexerSymbol::Minus, "-".into()))
        );
    }

    #[test]
    fn test_exact_match_multi_char() {
        let trie = make_trie();
        assert_eq!(
            match_symbol(&trie, "=="),
            Some((LexerSymbol::EqualEqual, "==".into()))
        );
    }

    #[test]
    fn test_longest_match() {
        let trie = make_trie();

        let mut chars = "==foo".chars().peekable();
        let result = trie.match_longest(&mut chars);
        assert_eq!(result, Some((LexerSymbol::EqualEqual, "==".into())));
        assert_eq!(chars.collect::<String>(), "foo"); // input advanced

        let mut chars = "+=bar".chars().peekable();
        let result = trie.match_longest(&mut chars);
        assert_eq!(result, Some((LexerSymbol::PlusEqual, "+=".into())));
        assert_eq!(chars.collect::<String>(), "bar");
    }

    #[test]
    fn test_ambiguous_prefix() {
        let trie = make_trie();

        let mut chars = "=baz".chars().peekable();
        let result = trie.match_longest(&mut chars);
        assert_eq!(result, Some((LexerSymbol::Equal, "=".into())));
        assert_eq!(chars.collect::<String>(), "baz");
    }

    #[test]
    fn test_no_match() {
        let trie = make_trie();

        let mut chars = "?invalid".chars().peekable();
        let result = trie.match_longest(&mut chars);
        assert_eq!(result, None);
        assert_eq!(chars.collect::<String>(), "?invalid"); // input unchanged
    }

    #[test]
    fn test_partial_match_not_accepted() {
        let trie = make_trie();

        let mut chars = "+-foo".chars().peekable(); // "+-" not in trie
        let result = trie.match_longest(&mut chars);
        assert_eq!(result, Some((LexerSymbol::Plus, "+".into()))); // only "+" matched
        assert_eq!(chars.collect::<String>(), "-foo"); // "+" consumed
    }
}
