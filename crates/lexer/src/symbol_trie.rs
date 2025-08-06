use std::{iter::Peekable, str::Chars};

use crate::{tokens::LexerSymbol, trie::Trie};

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolTrie {
    trie: Trie<LexerSymbol>,
}

impl Default for SymbolTrie {
    fn default() -> Self {
        let mut trie = Trie::<LexerSymbol>::default();
        trie.insert("=", LexerSymbol::Equal);
        trie.insert("==", LexerSymbol::EqualEqual);
        trie.insert("->", LexerSymbol::Arrow);
        trie.insert("+", LexerSymbol::Plus);
        trie.insert("-", LexerSymbol::Minus);
        trie.insert("*", LexerSymbol::Multiply);
        trie.insert("/", LexerSymbol::Divide);
        trie.insert("*=", LexerSymbol::MultiplyEqual);
        trie.insert("/=", LexerSymbol::DivideEqual);
        trie.insert("+=", LexerSymbol::PlusEqual);
        trie.insert("-=", LexerSymbol::MinusEqual);
        trie.insert("!=", LexerSymbol::NotEqual);
        trie.insert(";", LexerSymbol::SemiColon);
        Self { trie }
    }
}

impl SymbolTrie {
    pub fn insert(&mut self, s: &str, sym: LexerSymbol) {
        self.trie.insert(s, sym)
    }
    pub fn match_longest(&self, chars: &mut Peekable<Chars>) -> Option<(LexerSymbol, String)> {
        self.trie.match_longest(chars)
    }
}
