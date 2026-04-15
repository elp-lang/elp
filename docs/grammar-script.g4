grammar Elp;

/*
 * Elp scripting grammar — .elp files only.
 *
 * Role: this grammar is a specification and test rig, not a code generator
 *       for the compiler. The production lexer and parser live in
 *       crates/lexer and crates/parser and are hand-written. This .g4 is
 *       for:
 *         - railroad diagrams (ANTLRWorks, antlr4-tools, IntelliJ ANTLR)
 *         - ambiguity reports at grammar-compile time
 *         - `grun Elp script -tree path/to/file.elp` to check shapes
 *         - diffing against the hand-written parser's AST when something
 *           looks off
 *
 * Scope: the script dialect only. The view dialect (.velp) and stylesheet
 *        dialect (.selp) layer on top of this and are not defined here.
 *
 * Known simplifications vs the hand-written parser — see the "Caveats"
 * section at the bottom of the file before arguing with a parse result.
 */


// =============================================================
// Parser rules (lowercase = parser, UPPERCASE = lexer)
// =============================================================

script
    : item* EOF
    ;

item
    : importStmt
    | macroAttr
    | declaration
    | expr
    ;


// ---------- Imports ----------

importStmt
    : IMPORT '{' importList '}' FROM modulePath
    ;

importList
    : importName (',' importName)* ','?
    ;

importName
    : IDENT (AS IDENT)?
    ;

// A leading '.' marks a project-root-relative import.
modulePath
    : '.'? IDENT ('.' IDENT)*
    ;


// ---------- Declarations ----------

declaration
    : macroAttr* EXPORT? declBody
    ;

declBody
    : fnDecl
    | varDecl
    | constDecl
    | objectDecl
    | interfaceDecl
    | enumDecl
    ;


// ---------- Macros ----------

// @Name, @Name<Type,...>, @Name(args), or @Name<Type,...>(args).
// Macros may appear in the generic-arg position of other macros.
macroAttr
    : '@' IDENT genericArgs? ('(' argList? ')')?
    ;


// ---------- Functions ----------

// Canonical shapes:
//   fn Name { ... }
//   fn Name(params) { ... }
//   fn Name -> Ret { ... }
//   fn Name(params) -> Ret { ... }
//   fn Type.method(params) -> Ret { ... }
// Plus an optional `pure` prefix.
fnDecl
    : PURE? FN fnName genericParams?
      ('(' paramList? ')')?
      ('->' type)?
      block
    ;

fnName
    : IDENT ('.' IDENT)?
    ;

paramList
    : param (',' param)* ','?
    ;

param
    : selfParam
    | CONST? IDENT type ('=' expr)?
    | CONST? type
    ;

selfParam
    : '&'? SELF
    ;


// ---------- Variables ----------

// var = mutable binding (Rust `let mut`), const = immutable (Rust `let`).
varDecl
    : VAR varPattern type? ('=' expr)?
    ;

constDecl
    : CONST varPattern type? ('=' expr)?
    ;

// Multi-return destructuring: `const result, err = getThing()`.
varPattern
    : IDENT (',' IDENT)*
    ;


// ---------- Objects ----------

objectDecl
    : OBJECT IDENT genericParams?
      (IMPLEMENTS typeList)?
      '{' fieldList '}'
    ;

fieldList
    : (field (',' field)* ','?)?
    ;

// Three forms:
//   name Type                typed, no default
//   name Type = Expr         typed with default
//   name = Expr              type inferred from default
// Any form may carry an optional struct tag.
field
    : IDENT type ('=' expr)? structTag?
    | IDENT '=' expr structTag?
    ;

structTag
    : BACKTICK_STRING
    ;


// ---------- Interfaces ----------

// Interfaces may declare required fields AND method signatures.
// Method signatures reuse the fnDecl header form without a body.
interfaceDecl
    : INTERFACE IDENT genericParams? '{' interfaceBody '}'
    ;

interfaceBody
    : (interfaceMember (',' interfaceMember)* ','?)?
    ;

interfaceMember
    : field
    | FN IDENT genericParams? ('(' paramList? ')')? ('->' type)?
    ;


// ---------- Enums ----------

enumDecl
    : ENUM IDENT genericParams? '{' enumVariants '}'
    ;

enumVariants
    : (variantEntry (',' variantEntry)* ','?)?
    ;

variantEntry
    : macroAttr* variant
    ;

variant
    : IDENT ('(' typeList ')')?
    ;


// ---------- Generics ----------

genericParams
    : '<' genericParam (',' genericParam)* '>'
    ;

genericParam
    : IDENT (':' typeBounds)?
    ;

typeBounds
    : type ('+' type)*
    ;

genericArgs
    : '<' typeOrMacro (',' typeOrMacro)* '>'
    ;

typeOrMacro
    : type
    | macroAttr
    ;


// ---------- Types ----------

// Left-recursive rule — ANTLR4 handles this natively.
type
    : type '?'                       // OptionalType
    | '&' type                       // RefType
    | '[' type ']'                   // ArrayType
    | OBJECT '{' fieldList '}'       // AnonObjectType
    | namedType                      // base case
    ;

namedType
    : IDENT ('.' IDENT)* genericArgs?
    ;

typeList
    : type (',' type)*
    ;


// ---------- Blocks and statements ----------

block
    : '{' statement* '}'
    ;

// Statements are newline-separated in the real parser; here they're
// greedy-parsed (see Caveats).
statement
    : varDecl
    | constDecl
    | assignment
    | returnStmt
    | ifExpr
    | matchExpr
    | forStmt
    | exprStmt
    ;

exprStmt
    : expr
    ;

returnStmt
    : RETURN expr?
    ;

assignment
    : lvalue assignOp expr
    ;

assignOp
    : '=' | '+=' | '-=' | '*=' | '/=' | '&=' | '|='
    ;

lvalue
    : IDENT (memberAccess | index)*
    ;


// ---------- Control flow ----------

// if and match are expressions — they can appear in return position
// or be assigned to a binding.
ifExpr
    : IF expr block (ELSE (ifExpr | block))?
    ;

matchExpr
    : MATCH expr '{' matchArms '}'
    ;

matchArms
    : (matchArm (',' matchArm)* ','?)?
    ;

matchArm
    : pattern '->' armBody
    ;

armBody
    : expr
    | block
    ;

pattern
    : literal
    | WILDCARD
    | rangePat
    | variantPat
    | tuplePat
    | IDENT                          // binding pattern
    ;

// Range pattern — literals only on both sides in practice,
// but the grammar accepts any expr for spec flexibility.
rangePat
    : expr '..' expr
    ;

variantPat
    : '.' IDENT ('(' patternList ')')?     // contextual .Variant
    | IDENT '(' patternList ')'            // qualified Variant(x, y)
    ;

tuplePat
    : '(' pattern (',' pattern)+ ')'
    ;

patternList
    : pattern (',' pattern)*
    ;

forStmt
    : FOR IDENT IN expr block
    ;


// ---------- Expressions ----------

// ANTLR4 handles precedence via alternative order within a single
// left-recursive rule. Order here is high-precedence first
// (closest to leaves); ANTLR inverts this for left-recursion climbing.
expr
    : expr postfixOp                                       // postfix: a.b, a(), a[i], a { ... }
    | ('!' | '-' | '&') expr                               // unary
    | expr op=('*' | '/' | '%') expr                       // multiplicative
    | expr op=('+' | '-') expr                             // additive
    | expr op=('<' | '>' | '<=' | '>=') expr               // relational
    | expr op=('==' | '!=') expr                           // equality
    | expr '&&' expr                                       // logical and
    | expr '||' expr                                       // logical or
    | primary                                              // atom
    ;

postfixOp
    : memberAccess
    | call
    | index
    | builderBlock
    ;

memberAccess
    : '.' IDENT
    ;

call
    : genericArgs? '(' argList? ')'
    ;

index
    : '[' expr ']'
    ;

// Trailing '{ ... }' after a call, used to attach children:
// `App { Surface { Content() } }`. Rare in .elp, central in .velp.
builderBlock
    : '{' statement* '}'
    ;

primary
    : literal
    | contextualVariant
    | IDENT
    | SELF
    | objectLiteral
    | arrayLiteral
    | tupleLiteral
    | lambdaExpr
    | matchExpr
    | ifExpr
    | '(' expr ')'
    ;

contextualVariant
    : '.' IDENT
    ;

// Typed:    `Type { field = value, ... }`
// Anon:     `{ field = value, ... }`  (type inferred from context)
objectLiteral
    : namedType? '{' (fieldInit (',' fieldInit)* ','?)? '}'
    ;

fieldInit
    : IDENT '=' expr
    | IDENT                         // shorthand: name is key and value
    ;

arrayLiteral
    : '[' (expr (',' expr)* ','?)? ']'
    ;

// Two or more comma-separated exprs. `(x)` alone is a grouped expression.
tupleLiteral
    : '(' expr (',' expr)+ ','? ')'
    ;

// Same shapes as fnDecl minus the name.
lambdaExpr
    : FN ('(' paramList? ')')? ('->' type)? block
    ;

argList
    : arg (',' arg)* ','?
    ;

arg
    : (IDENT '=')? expr             // named or positional
    ;

literal
    : INTEGER
    | FLOAT
    | STRING
    | TRUE
    | FALSE
    | NIL
    ;


// =============================================================
// Lexer rules
// =============================================================

// ---------- Keywords (MUST come before IDENT for longest-match) ----------

IMPORT     : 'import' ;
FROM       : 'from' ;
AS         : 'as' ;
EXPORT     : 'export' ;
FN         : 'fn' ;
PURE       : 'pure' ;
VAR        : 'var' ;
CONST      : 'const' ;
OBJECT     : 'object' ;
INTERFACE  : 'interface' ;
ENUM       : 'enum' ;
IMPLEMENTS : 'implements' ;
IF         : 'if' ;
ELSE       : 'else' ;
MATCH      : 'match' ;
FOR        : 'for' ;
IN         : 'in' ;
RETURN     : 'return' ;
SELF       : 'self' ;
TRUE       : 'true' ;
FALSE      : 'false' ;
NIL        : 'nil' ;


// ---------- Punctuation explicitly named for readability ----------
// (String literals in parser rules auto-generate anonymous tokens;
//  these give a few of the trickier ones real names.)

WILDCARD    : '_' ;
RANGE       : '..' ;


// ---------- Identifiers and literals ----------

// Simplified ASCII identifier — the real lexer uses XID_Start / XID_Continue
// and accepts Unicode identifiers including emoji. This is fine for a spec.
IDENT       : [a-zA-Z_] [a-zA-Z0-9_]* ;

INTEGER     : [0-9] [0-9_]* ;

// Float requires at least one digit on both sides of the '.', so `400..499`
// still lexes as INTEGER RANGE INTEGER rather than eating the range operator.
FLOAT       : [0-9] [0-9_]* '.' [0-9] [0-9_]* ;

// Opaque string literal. Swift/Kotlin style interpolation (`{ expr }` inside
// "...") is NOT modeled here — see Caveats.
STRING      : '"' ( ~["\\] | '\\' . )* '"' ;

// Struct tag: backtick-delimited metadata, content is opaque to the grammar.
BACKTICK_STRING : '`' ~[`]* '`' ;


// ---------- Trivia ----------

// DOC_COMMENT must precede LINE_COMMENT — both match '//...' but DOC_COMMENT
// requires a third slash and ANTLR uses first-defined wins on length ties.
DOC_COMMENT   : '///' ~[\r\n]*           -> channel(HIDDEN) ;
LINE_COMMENT  : '//'  ~[\r\n]*           -> skip ;
BLOCK_COMMENT : '/*' .*? '*/'            -> skip ;

// Newlines are hidden for the spec grammar. The real parser treats them
// as significant statement/list separators — see Caveats.
NEWLINE       : ('\r'? '\n' | '\r')      -> channel(HIDDEN) ;

WS            : [ \t]+                   -> skip ;


/* =============================================================
   Caveats — known divergences from the hand-written parser
   =============================================================

   1. NEWLINE significance is not modeled.
      The hand-written lexer in crates/lexer emits NEWLINE as a real token
      and the parser treats them as statement/field/match-arm separators.
      This grammar pushes NEWLINE to the hidden channel and uses commas as
      the canonical list separator. Consequence: this grammar accepts a
      superset of valid Elp — every valid .elp file parses cleanly here,
      but a few things this grammar accepts would be rejected by the
      real parser (e.g. `x = 1 y = 2` on one line with no separator).

   2. String interpolation is not modeled.
      STRING is lexed opaquely as '"' ... '"'. The real design is Swift/
      Kotlin-style `{ expr }` inside string literals, which requires lexer
      modes (pushMode/popMode). Added when it becomes load-bearing.

   3. Contextual keywords are fully reserved.
      `from` and `as` are keyword tokens here, which means they cannot
      appear as identifiers anywhere. The real design may (or may not)
      want them to be contextual, usable as parameter names outside
      import context. If contextual, the g4 needs semantic predicates
      or an `ident` meta-rule that unions IDENT with those keyword tokens.

   4. Identifiers are ASCII-only.
      Real lexer uses XID_Start / XID_Continue plus emoji presentation.
      The spec grammar uses `[a-zA-Z_][a-zA-Z0-9_]*` for tractability.

   5. Statement-or-expression-builder-block ambiguity.
      Both `block` and `builderBlock` are `{ statement* }`. A trailing
      `{ ... }` after a primary expression is parsed as a builderBlock;
      the same construct at statement position parses as a block.
      ANTLR's adaptive prediction resolves this in favor of the longest
      parse, which matches the real parser's intent but isn't formally
      guaranteed — verify with `grun` if you hit a surprising tree.

   6. If-condition-then-builder-block ambiguity.
      `if foo { ... } else { ... }` is ambiguous if `foo` contains a
      builderBlock of its own. Rust solves this by disallowing struct
      literals in `if` conditions; Elp's solution is TBD. The grammar
      currently accepts both parses.

   7. Floats vs ranges.
      `400..499` lexes as INTEGER RANGE INTEGER because FLOAT requires
      digits on both sides of the dot. `400.5..499.0` also works.
      `.5..1.0` does NOT work — FLOAT requires a leading digit.

   8. Nested function declarations are syntactically allowed inside
      `block`, but are rejected by the real parser's semantic pass.
      Keeping them in the grammar lets the parser produce a specific
      error message rather than a generic one.
   ============================================================= */
