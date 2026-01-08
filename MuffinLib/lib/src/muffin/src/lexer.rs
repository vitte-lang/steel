//! lexer.rs 
//!
//! Lexer pour Muffin Bakefile v2 (Muffinfile / build.muf).
//!
//! Couverture EBNF (résumé):
//! - comments: `# ...` (line comment)
//! - header: `muffin bake <int>`
//! - keywords: store, capsule, var, profile, tool, bake, wire, export, plan, switch, set,
//!             path, mode, env, fs, net, time, allow, deny, allow_read, allow_write,
//!             allow_write_exact, stable, exec, expect_version, sandbox, in, out, make,
//!             glob, file, text, value, run, takes, emits, cache, output, at, flag, as,
//!             exports, true, false
//! - literals: int, string (escapes: \", \\, \n, \r, \t)
//! - punctuation: .end, :, =, ->, ., ,, [, ], (legacy: { } pas utilisé), newline
//!
//! Contraintes : std uniquement.
//!
//! Intégration diag:
//! - Le lexer émet des tokens avec Span (crate::diag::Span).
//! - En cas d’erreur, ajoute un Diagnostic dans DiagBag, mais continue (recover).
//!
//! API:
//! - Lexer::new(file_id, &str) -> Lexer
//! - lexer.lex_all(&mut DiagBag) -> Vec<Token>
//!
//! Remarque : la grammaire est orientée lignes, mais le lexer est whitespace-tolérant.
//! On expose des tokens Newline pour permettre au parser d’implémenter des règles
//! “line-based” si nécessaire.

use std::ops::Range;

use crate::diag::{DiagBag, Diagnostic, Span};

/// ------------------------------------------------------------
/// Token model
/// ------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // structural
    Eof,
    Newline,

    // identifiers / literals
    Ident,
    Int,
    String,

    // keywords (subset utile)
    KwMuffin,
    KwBake,

    KwStore,
    KwCapsule,
    KwVar,
    KwProfile,
    KwTool,
    KwBakeBlock,
    KwWire,
    KwExport,
    KwPlan,
    KwSwitch,
    KwSet,

    KwPath,
    KwMode,
    KwEnv,
    KwFs,
    KwNet,
    KwTime,
    KwAllow,
    KwDeny,
    KwAllowRead,
    KwAllowWrite,
    KwAllowWriteExact,
    KwStable,

    KwExec,
    KwExpectVersion,
    KwSandbox,

    KwIn,
    KwOut,
    KwMake,
    KwGlob,
    KwFile,
    KwText,
    KwValue,

    KwRun,
    KwToolRef, // `tool` après `run` (optionnel côté parser)
    KwTakes,
    KwEmits,
    KwAs,

    KwCache,
    KwOutput,
    KwAt,

    KwFlag,
    KwExports,

    KwTrue,
    KwFalse,

    // punctuation/operators
    Dot,        // .
    Colon,      // :
    Eq,         // =
    Comma,      // ,
    LBracket,   // [
    RBracket,   // ]
    Arrow,      // ->
    DotEnd,     // .end

    // errors / unknown
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    /// Lexeme optionnel: utilisé pour Ident/String/Int (et Unknown).
    pub text: Option<String>,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span, text: None }
    }
    pub fn with_text(mut self, s: String) -> Self {
        self.text = Some(s);
        self
    }
}

/// ------------------------------------------------------------
/// Lexer
/// ------------------------------------------------------------

#[derive(Debug)]
pub struct Lexer<'a> {
    file_id: u32,
    src: &'a str,
    bytes: &'a [u8],
    i: usize,
    len: usize,

    /// Option: si true, émettre Newline tokens (sinon, les consommer comme ws).
    emit_newlines: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(file_id: u32, src: &'a str) -> Self {
        Self {
            file_id,
            src,
            bytes: src.as_bytes(),
            i: 0,
            len: src.len(),
            emit_newlines: true,
        }
    }

    pub fn emit_newlines(mut self, on: bool) -> Self {
        self.emit_newlines = on;
        self
    }

    pub fn lex_all(mut self, diags: &mut DiagBag) -> Vec<Token> {
        let mut out = Vec::new();
        loop {
            let t = self.next_token(diags);
            let k = t.kind;
            out.push(t);
            if k == TokenKind::Eof {
                break;
            }
        }
        out
    }

    fn next_token(&mut self, diags: &mut DiagBag) -> Token {
        // skip spaces/tabs + comments; optionally newlines
        loop {
            if self.i >= self.len {
                return Token::new(TokenKind::Eof, self.span(self.i..self.i));
            }

            let b = self.bytes[self.i];

            // whitespace
            if b == b' ' || b == b'\t' {
                self.i += 1;
                continue;
            }

            // newline
            if b == b'\n' {
                let start = self.i;
                self.i += 1;
                if self.emit_newlines {
                    return Token::new(TokenKind::Newline, self.span(start..self.i));
                }
                continue;
            }
            if b == b'\r' {
                let start = self.i;
                self.i += 1;
                if self.i < self.len && self.bytes[self.i] == b'\n' {
                    self.i += 1;
                }
                if self.emit_newlines {
                    return Token::new(TokenKind::Newline, self.span(start..self.i));
                }
                continue;
            }

            // comment (# ... to end-of-line)
            if b == b'#' {
                self.skip_comment();
                continue;
            }

            break;
        }

        let start = self.i;
        let b = self.bytes[self.i];

        // .end
        if b == b'.' {
            if self.starts_with_bytes(b".end") {
                self.i += 4;
                return Token::new(TokenKind::DotEnd, self.span(start..self.i));
            }
            self.i += 1;
            return Token::new(TokenKind::Dot, self.span(start..self.i));
        }

        // ->
        if b == b'-' && self.peek_byte(1) == Some(b'>') {
            self.i += 2;
            return Token::new(TokenKind::Arrow, self.span(start..self.i));
        }

        // punctuation
        match b {
            b':' => {
                self.i += 1;
                return Token::new(TokenKind::Colon, self.span(start..self.i));
            }
            b'=' => {
                self.i += 1;
                return Token::new(TokenKind::Eq, self.span(start..self.i));
            }
            b',' => {
                self.i += 1;
                return Token::new(TokenKind::Comma, self.span(start..self.i));
            }
            b'[' => {
                self.i += 1;
                return Token::new(TokenKind::LBracket, self.span(start..self.i));
            }
            b']' => {
                self.i += 1;
                return Token::new(TokenKind::RBracket, self.span(start..self.i));
            }
            b'"' => return self.lex_string(diags),
            _ => {}
        }

        // int
        if is_digit(b) {
            return self.lex_int();
        }

        // ident / keyword
        if is_ident_start(b) {
            return self.lex_ident_or_kw();
        }

        // unknown char
        self.i += 1;
        let span = self.span(start..self.i);
        diags.push(Diagnostic::error(format!("unexpected character `{}`", byte_as_char(b))).with_span(span));
        Token::new(TokenKind::Unknown, span).with_text(byte_as_char(b).to_string())
    }

    fn lex_int(&mut self) -> Token {
        let start = self.i;
        while self.i < self.len && is_digit(self.bytes[self.i]) {
            self.i += 1;
        }
        let span = self.span(start..self.i);
        let text = self.src[start..self.i].to_string();
        Token::new(TokenKind::Int, span).with_text(text)
    }

    fn lex_ident_or_kw(&mut self) -> Token {
        let start = self.i;
        self.i += 1;
        while self.i < self.len && is_ident_continue(self.bytes[self.i]) {
            self.i += 1;
        }
        let span = self.span(start..self.i);
        let text = &self.src[start..self.i];

        let kind = keyword_kind(text).unwrap_or(TokenKind::Ident);
        let mut t = Token::new(kind, span);
        if kind == TokenKind::Ident {
            t.text = Some(text.to_string());
        }
        t
    }

    fn lex_string(&mut self, diags: &mut DiagBag) -> Token {
        let start = self.i;
        self.i += 1; // consume '"'
        let mut out = String::new();

        while self.i < self.len {
            let b = self.bytes[self.i];

            // end
            if b == b'"' {
                self.i += 1;
                let span = self.span(start..self.i);
                return Token::new(TokenKind::String, span).with_text(out);
            }

            // newline in string => error + recover
            if b == b'\n' || b == b'\r' {
                let span = self.span(start..self.i);
                diags.push(Diagnostic::error("unterminated string literal").with_span(span));
                // do not consume newline here; let outer loop handle
                return Token::new(TokenKind::String, span).with_text(out);
            }

            // escape
            if b == b'\\' {
                if self.i + 1 >= self.len {
                    let span = self.span(start..self.i + 1);
                    diags.push(Diagnostic::error("unterminated escape sequence").with_span(span));
                    self.i += 1;
                    let span2 = self.span(start..self.i);
                    return Token::new(TokenKind::String, span2).with_text(out);
                }
                let esc = self.bytes[self.i + 1];
                match esc {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    _ => {
                        let sp = self.span(self.i..self.i + 2);
                        diags.push(Diagnostic::warning(format!(
                            "unknown escape sequence: \\{}",
                            byte_as_char(esc)
                        ))
                        .with_span(sp));
                        out.push(byte_as_char(esc));
                    }
                }
                self.i += 2;
                continue;
            }

            // regular
            out.push(byte_as_char(b));
            self.i += 1;
        }

        // EOF reached
        let span = self.span(start..self.i);
        diags.push(Diagnostic::error("unterminated string literal").with_span(span));
        Token::new(TokenKind::String, span).with_text(out)
    }

    fn skip_comment(&mut self) {
        // consume until \n or \r or eof
        while self.i < self.len {
            let b = self.bytes[self.i];
            if b == b'\n' || b == b'\r' {
                break;
            }
            self.i += 1;
        }
    }

    fn span(&self, r: Range<usize>) -> Span {
        Span::new(self.file_id, r.start as u32, r.end as u32)
    }

    fn peek_byte(&self, n: usize) -> Option<u8> {
        let j = self.i + n;
        if j < self.len { Some(self.bytes[j]) } else { None }
    }

    fn starts_with_bytes(&self, pat: &[u8]) -> bool {
        self.bytes.get(self.i..self.i + pat.len()).map(|s| s == pat).unwrap_or(false)
    }
}

/// ------------------------------------------------------------
/// Keyword map
/// ------------------------------------------------------------

fn keyword_kind(s: &str) -> Option<TokenKind> {
    Some(match s {
        "muffin" => TokenKind::KwMuffin,
        "bake" => TokenKind::KwBake,

        "store" => TokenKind::KwStore,
        "capsule" => TokenKind::KwCapsule,
        "var" => TokenKind::KwVar,
        "profile" => TokenKind::KwProfile,
        "tool" => TokenKind::KwTool,
        "bake" => TokenKind::KwBakeBlock,
        "wire" => TokenKind::KwWire,
        "export" => TokenKind::KwExport,
        "plan" => TokenKind::KwPlan,
        "switch" => TokenKind::KwSwitch,
        "set" => TokenKind::KwSet,

        "path" => TokenKind::KwPath,
        "mode" => TokenKind::KwMode,
        "env" => TokenKind::KwEnv,
        "fs" => TokenKind::KwFs,
        "net" => TokenKind::KwNet,
        "time" => TokenKind::KwTime,
        "allow" => TokenKind::KwAllow,
        "deny" => TokenKind::KwDeny,
        "allow_read" => TokenKind::KwAllowRead,
        "allow_write" => TokenKind::KwAllowWrite,
        "allow_write_exact" => TokenKind::KwAllowWriteExact,
        "stable" => TokenKind::KwStable,

        "exec" => TokenKind::KwExec,
        "expect_version" => TokenKind::KwExpectVersion,
        "sandbox" => TokenKind::KwSandbox,

        "in" => TokenKind::KwIn,
        "out" => TokenKind::KwOut,
        "make" => TokenKind::KwMake,
        "glob" => TokenKind::KwGlob,
        "file" => TokenKind::KwFile,
        "text" => TokenKind::KwText,
        "value" => TokenKind::KwValue,

        "run" => TokenKind::KwRun,
        "takes" => TokenKind::KwTakes,
        "emits" => TokenKind::KwEmits,
        "as" => TokenKind::KwAs,
        "cache" => TokenKind::KwCache,
        "output" => TokenKind::KwOutput,
        "at" => TokenKind::KwAt,

        "flag" => TokenKind::KwFlag,
        "exports" => TokenKind::KwExports,

        "true" => TokenKind::KwTrue,
        "false" => TokenKind::KwFalse,

        _ => return None,
    })
}

/// ------------------------------------------------------------
/// Char helpers
/// ------------------------------------------------------------

fn is_digit(b: u8) -> bool {
    b'0' <= b && b <= b'9'
}

fn is_ident_start(b: u8) -> bool {
    (b'A' <= b && b <= b'Z') || (b'a' <= b && b <= b'z') || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    is_ident_start(b) || is_digit(b)
}

fn byte_as_char(b: u8) -> char {
    if b.is_ascii() { b as char } else { '\u{FFFD}' }
}

/// ------------------------------------------------------------
/// Token stream convenience
/// ------------------------------------------------------------

#[derive(Debug)]
pub struct TokenStream {
    pub toks: Vec<Token>,
    pub i: usize,
}

impl TokenStream {
    pub fn new(toks: Vec<Token>) -> Self {
        Self { toks, i: 0 }
    }

    pub fn peek(&self) -> &Token {
        self.toks.get(self.i).unwrap_or_else(|| self.toks.last().unwrap())
    }

    pub fn next(&mut self) -> &Token {
        let t = self.peek();
        if t.kind != TokenKind::Eof {
            self.i += 1;
        }
        t
    }

    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            self.next();
            true
        } else {
            false
        }
    }
}

/// ------------------------------------------------------------
/// Tests
/// ------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_header_and_end() {
        let src = "muffin bake 2\nstore cache\npath \"./x\"\n.end\n";
        let mut diags = DiagBag::new();
        let toks = Lexer::new(0, src).lex_all(&mut diags);
        assert!(toks.iter().any(|t| t.kind == TokenKind::KwMuffin));
        assert!(toks.iter().any(|t| t.kind == TokenKind::KwBake));
        assert!(toks.iter().any(|t| t.kind == TokenKind::DotEnd));
        assert!(!diags.has_error());
    }

    #[test]
    fn lex_string_escapes() {
        let src = "\"a\\\\b\\n\\t\\\"c\"";
        let mut diags = DiagBag::new();
        let toks = Lexer::new(0, src).emit_newlines(false).lex_all(&mut diags);
        let s = toks.iter().find(|t| t.kind == TokenKind::String).unwrap().text.clone().unwrap();
        assert_eq!(s, "a\\b\n\t\"c");
    }

    #[test]
    fn lex_comment() {
        let src = "# c\nmuffin bake 2\n";
        let mut diags = DiagBag::new();
        let toks = Lexer::new(0, src).lex_all(&mut diags);
        assert!(toks.iter().any(|t| t.kind == TokenKind::KwMuffin));
    }
}