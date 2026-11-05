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
        trie.insert("{", LexerSymbol::LCurley);
        trie.insert("}", LexerSymbol::RCurley);
        trie.insert(".", LexerSymbol::Period);
        trie.insert("..", LexerSymbol::Range);
        trie.insert(",", LexerSymbol::Comma);
        trie.insert(":", LexerSymbol::Colon);
        trie.insert("@", LexerSymbol::At);
        trie.insert("<", LexerSymbol::LessThan);
        trie.insert(">", LexerSymbol::GreaterThan);
        trie.insert("<=", LexerSymbol::LessEqual);
        trie.insert(">=", LexerSymbol::GreaterEqual);
        trie.insert("&&", LexerSymbol::AndAnd);
        trie.insert("||", LexerSymbol::OrOr);
        trie.insert("&=", LexerSymbol::AmpersandEqual);
        trie.insert("|=", LexerSymbol::PipeEqual);
        trie.insert("%", LexerSymbol::Percent);
        trie.insert("&", LexerSymbol::Ampersand);
        trie.insert("!", LexerSymbol::Bang);
        trie.insert("?", LexerSymbol::Question);
        trie.insert("(", LexerSymbol::LParen);
        trie.insert(")", LexerSymbol::RParen);
        trie.insert("[", LexerSymbol::LBracket);
        trie.insert("]", LexerSymbol::RBracket);

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
