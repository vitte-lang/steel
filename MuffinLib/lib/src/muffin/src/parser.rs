//! parser.rs 
//!
//! Parser Muffin Bakefile v2 (Muffinfile / build.muf)
//!
//! Objectif : transformer les tokens en AST minimal (non résolu), puis fournir un
//! “bridge” vers HIR via un resolver (hors scope ici). Ce parser est robuste :
//! il produit des diagnostics et tente de continuer.
//!
//! Contrainte : std uniquement.
//!
//! Couverture EBNF (cible) :
//! - header: `muffin bake <int>`
//! - stmt: store/capsule/var/profile/tool/bake/wire/export/plan/switch/set
//! - blocks: top-level blocks terminés par `.end`
//! - valeurs: string/int/bool/list/ident
//!
//! Intégration :
//! - utilise `lexer::{Token, TokenKind, TokenStream}`
//! - utilise `diag::{DiagBag, Diagnostic, Span}`
//! - AST propre au parser (ci-dessous)
//!
//! Remarque : HIR “MAX” existe déjà. Ce parser peut être remplacé par un parser
//! plus strict. Ici, on privilégie une base solide.

use std::collections::BTreeMap;

use crate::diag::{DiagBag, Diagnostic, Span};
use crate::hir::{Interner, NameId, PrimType, TypeRef, Value};
use crate::lexer::{Token, TokenKind, TokenStream};

/// ------------------------------------------------------------
/// AST
/// ------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AstFile {
    pub header: AstHeader,
    pub stmts: Vec<AstStmt>,
}

#[derive(Debug, Clone)]
pub struct AstHeader {
    pub version: u32,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstStmt {
    Set(AstSet),

    Store(AstStore),
    Capsule(AstCapsule),

    Var(AstVar),

    Profile(AstProfile),
    Tool(AstTool),

    Bake(AstBake),

    Wire(AstWire),
    Export(AstExport),

    Plan(AstPlan),
    Switch(AstSwitch),
}

#[derive(Debug, Clone)]
pub struct AstSet {
    pub key: NameId,
    pub value: AstValue,
    pub span: Span,
}

/// store <ident> ... .end
#[derive(Debug, Clone)]
pub struct AstStore {
    pub name: NameId,
    pub items: Vec<AstStoreItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstStoreItem {
    Path(NameId),     // string
    Mode(NameId),     // ident: content|mtime|off
}

/// capsule <ident> ... .end
#[derive(Debug, Clone)]
pub struct AstCapsule {
    pub name: NameId,
    pub items: Vec<AstCapsuleItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstCapsuleItem {
    Env { kind: NameId, list: Vec<NameId>, span: Span }, // allow/deny + ["A","B"]
    Fs { kind: NameId, list: Vec<NameId>, span: Span },  // allow_read/allow_write/deny/allow_write_exact
    Net { kind: NameId, span: Span },                    // allow/deny
    TimeStable { value: bool, span: Span },              // stable true/false
}

/// var <ident> : <type> = <value>
#[derive(Debug, Clone)]
pub struct AstVar {
    pub name: NameId,
    pub ty: AstTypeRef,
    pub value: AstValue,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstTypeRef {
    Prim(PrimType),
    Artifact(Vec<NameId>), // ident.ident(.ident)*
}

/// profile <ident> ... .end
#[derive(Debug, Clone)]
pub struct AstProfile {
    pub name: NameId,
    pub items: Vec<AstProfileItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstProfileItem {
    pub key: NameId,
    pub value: AstValue,
    pub span: Span,
}

/// tool <ident> ... .end
#[derive(Debug, Clone)]
pub struct AstTool {
    pub name: NameId,
    pub items: Vec<AstToolItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstToolItem {
    Exec(NameId),
    ExpectVersion(NameId),
    Sandbox(bool),
    Capsule(NameId),
}

/// bake <ident> ... .end
#[derive(Debug, Clone)]
pub struct AstBake {
    pub name: NameId,
    pub items: Vec<AstBakeItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstBakeItem {
    InPort { name: NameId, ty: AstTypeRef, span: Span },
    OutPort { name: NameId, ty: AstTypeRef, span: Span },

    Make { name: NameId, kind: NameId, arg: NameId, span: Span }, // kind is ident (glob/file/text/value)
    Run(AstRunBlock),
    Cache { mode: NameId, span: Span }, // content|mtime|off
    OutputAt { port: NameId, at: NameId, span: Span },
}

/// run tool <ident> ... .end
#[derive(Debug, Clone)]
pub struct AstRunBlock {
    pub tool: NameId,
    pub items: Vec<AstRunItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstRunItem {
    Takes { port: NameId, flag: NameId, span: Span },
    Emits { port: NameId, flag: NameId, span: Span },
    Set { flag: NameId, value: AstValue, span: Span },
}

/// wire <ref> -> <ref>
#[derive(Debug, Clone)]
pub struct AstWire {
    pub from: AstRef,
    pub to: AstRef,
    pub span: Span,
}

/// export <ref>
#[derive(Debug, Clone)]
pub struct AstExport {
    pub what: AstRef,
    pub span: Span,
}

/// plan <ident> ... .end
#[derive(Debug, Clone)]
pub struct AstPlan {
    pub name: NameId,
    pub items: Vec<AstPlanItem>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstPlanItem {
    RunExports { span: Span },
    RunRef { what: AstRef, span: Span },
}

/// switch ... .end
#[derive(Debug, Clone)]
pub struct AstSwitch {
    pub flags: Vec<AstSwitchFlag>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AstSwitchFlag {
    pub flag: NameId, // string
    pub action: AstSwitchAction,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AstSwitchAction {
    Set { key: NameId, value: AstValue, span: Span },
    SetPlan { plan: NameId, span: Span },
    RunExports { span: Span },
    RunRef { what: AstRef, span: Span },
}

/// var or bake.port
#[derive(Debug, Clone)]
pub enum AstRef {
    Var(NameId),
    BakePort { bake: NameId, port: NameId, span: Span },
}

/// values
#[derive(Debug, Clone)]
pub enum AstValue {
    Str(NameId),
    Int(i64),
    Bool(bool),
    List(Vec<AstValue>),
    Ident(NameId),
}

/// ------------------------------------------------------------
/// Parser API
/// ------------------------------------------------------------

#[derive(Debug)]
pub struct Parser<'a> {
    ts: TokenStream,
    pub interner: &'a mut Interner,
    pub diags: &'a mut DiagBag,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, interner: &'a mut Interner, diags: &'a mut DiagBag) -> Self {
        Self { ts: TokenStream::new(tokens), interner, diags }
    }

    pub fn parse_file(&mut self) -> AstFile {
        let header = self.parse_header();
        let mut stmts = Vec::new();

        // skip possible newlines
        self.skip_nl();

        while self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }

            match self.parse_stmt() {
                Some(s) => stmts.push(s),
                None => {
                    // recover: skip line/block
                    self.recover_stmt();
                }
            }

            self.skip_nl();
        }

        AstFile { header, stmts }
    }

    /// header ::= "muffin" ws1 "bake" ws1 int_lit ;
    fn parse_header(&mut self) -> AstHeader {
        let start = self.ts.peek().span;
        self.expect(TokenKind::KwMuffin, "expected `muffin` header");
        self.expect(TokenKind::KwBake, "expected `bake` header");

        let tok = self.ts.peek().clone();
        let version = if tok.kind == TokenKind::Int {
            self.ts.next();
            tok.text.as_deref().and_then(|s| s.parse::<u32>().ok()).unwrap_or(0)
        } else {
            self.diags.push(Diagnostic::error("expected version integer after `muffin bake`").with_span(tok.span));
            0
        };

        // header ends at end-of-line or token sequence; accept newline or EOF
        let end = self.ts.peek().span;
        AstHeader { version, span: Span::join(start, end) }
    }

    fn parse_stmt(&mut self) -> Option<AstStmt> {
        match self.ts.peek().kind {
            TokenKind::KwSet => self.parse_set().map(AstStmt::Set),
            TokenKind::KwStore => self.parse_store().map(AstStmt::Store),
            TokenKind::KwCapsule => self.parse_capsule().map(AstStmt::Capsule),
            TokenKind::KwVar => self.parse_var().map(AstStmt::Var),
            TokenKind::KwProfile => self.parse_profile().map(AstStmt::Profile),
            TokenKind::KwTool => self.parse_tool().map(AstStmt::Tool),
            TokenKind::KwBakeBlock => self.parse_bake().map(AstStmt::Bake),
            TokenKind::KwWire => self.parse_wire().map(AstStmt::Wire),
            TokenKind::KwExport => self.parse_export().map(AstStmt::Export),
            TokenKind::KwPlan => self.parse_plan().map(AstStmt::Plan),
            TokenKind::KwSwitch => self.parse_switch().map(AstStmt::Switch),
            _ => {
                let sp = self.ts.peek().span;
                self.diags.push(Diagnostic::error("unexpected statement").with_span(sp));
                None
            }
        }
    }

    /// set_stmt ::= "set" ws1 ident ws1 value ;
    fn parse_set(&mut self) -> Option<AstSet> {
        let start = self.bump(TokenKind::KwSet)?;
        let key = self.parse_ident("expected identifier after `set`")?;
        let value = self.parse_value()?;
        Some(AstSet { key, value, span: Span::join(start, self.prev_span()) })
    }

    /// store_block ::= "store" ident nl { store_item nl } .end
    fn parse_store(&mut self) -> Option<AstStore> {
        let start = self.bump(TokenKind::KwStore)?;
        let name = self.parse_ident("expected store name")?;
        self.require_nl_or_eof("expected newline after store header");

        let mut items = Vec::new();
        while !self.at_end_block() && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }

            match self.ts.peek().kind {
                TokenKind::KwPath => {
                    let sp0 = self.bump(TokenKind::KwPath)?;
                    let s = self.parse_string("expected string after `path`")?;
                    items.push(AstStoreItem::Path(s));
                    self.end_stmt_line(sp0);
                }
                TokenKind::KwMode => {
                    let sp0 = self.bump(TokenKind::KwMode)?;
                    let m = self.parse_ident("expected mode (content|mtime|off)")?;
                    items.push(AstStoreItem::Mode(m));
                    self.end_stmt_line(sp0);
                }
                _ => {
                    let sp = self.ts.peek().span;
                    self.diags.push(Diagnostic::error("invalid store item").with_span(sp));
                    self.recover_line();
                }
            }
            self.skip_nl();
        }

        let end = self.end_block("expected `.end` to close store block")?;
        Some(AstStore { name, items, span: Span::join(start, end) })
    }

    /// capsule_block ::= "capsule" ident nl { capsule_item nl } .end
    fn parse_capsule(&mut self) -> Option<AstCapsule> {
        let start = self.bump(TokenKind::KwCapsule)?;
        let name = self.parse_ident("expected capsule name")?;
        self.require_nl_or_eof("expected newline after capsule header");

        let mut items = Vec::new();
        while !self.at_end_block() && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }

            match self.ts.peek().kind {
                TokenKind::KwEnv => {
                    let sp0 = self.bump(TokenKind::KwEnv)?;
                    let kind = self.parse_ident("expected env policy (allow|deny)")?;
                    let list = self.parse_list_string()?;
                    items.push(AstCapsuleItem::Env { kind, list, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                TokenKind::KwFs => {
                    let sp0 = self.bump(TokenKind::KwFs)?;
                    let kind = self.parse_ident("expected fs policy (allow_read|allow_write|deny|allow_write_exact)")?;
                    let list = self.parse_list_string()?;
                    items.push(AstCapsuleItem::Fs { kind, list, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                TokenKind::KwNet => {
                    let sp0 = self.bump(TokenKind::KwNet)?;
                    let kind = self.parse_ident("expected net policy (allow|deny)")?;
                    items.push(AstCapsuleItem::Net { kind, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                TokenKind::KwTime => {
                    let sp0 = self.bump(TokenKind::KwTime)?;
                    // time stable true/false
                    let _stable_kw = if self.ts.peek().kind == TokenKind::KwStable {
                        self.ts.next();
                        true
                    } else {
                        self.diags.push(Diagnostic::error("expected `stable` after `time`").with_span(self.ts.peek().span));
                        false
                    };
                    let b = self.parse_bool("expected boolean after `time stable`")?;
                    items.push(AstCapsuleItem::TimeStable { value: b, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                _ => {
                    self.diags.push(Diagnostic::error("invalid capsule item").with_span(self.ts.peek().span));
                    self.recover_line();
                }
            }
        }

        let end = self.end_block("expected `.end` to close capsule block")?;
        Some(AstCapsule { name, items, span: Span::join(start, end) })
    }

    /// var_decl ::= "var" ident ":" type_ref "=" value
    fn parse_var(&mut self) -> Option<AstVar> {
        let start = self.bump(TokenKind::KwVar)?;
        let name = self.parse_ident("expected var name")?;

        self.expect(TokenKind::Colon, "expected `:` after var name");
        let ty = self.parse_type_ref()?;

        self.expect(TokenKind::Eq, "expected `=` after var type");
        let value = self.parse_value()?;

        Some(AstVar { name, ty, value, span: Span::join(start, self.prev_span()) })
    }

    /// profile_block ::= "profile" ident nl { "set" ident value nl } .end
    fn parse_profile(&mut self) -> Option<AstProfile> {
        let start = self.bump(TokenKind::KwProfile)?;
        let name = self.parse_ident("expected profile name")?;
        self.require_nl_or_eof("expected newline after profile header");

        let mut items = Vec::new();
        while !self.at_end_block() && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }

            let sp0 = self.bump(TokenKind::KwSet)?;
            let key = self.parse_ident("expected key after set")?;
            let value = self.parse_value()?;
            items.push(AstProfileItem { key, value, span: Span::join(sp0, self.prev_span()) });
            self.skip_nl();
        }

        let end = self.end_block("expected `.end` to close profile block")?;
        Some(AstProfile { name, items, span: Span::join(start, end) })
    }

    /// tool_block ::= "tool" ident nl { tool_item nl } .end
    fn parse_tool(&mut self) -> Option<AstTool> {
        let start = self.bump(TokenKind::KwTool)?;
        let name = self.parse_ident("expected tool name")?;
        self.require_nl_or_eof("expected newline after tool header");

        let mut items = Vec::new();
        while !self.at_end_block() && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }

            match self.ts.peek().kind {
                TokenKind::KwExec => {
                    let sp0 = self.bump(TokenKind::KwExec)?;
                    let s = self.parse_string("expected string after `exec`")?;
                    items.push(AstToolItem::Exec(s));
                    self.end_stmt_line(sp0);
                }
                TokenKind::KwExpectVersion => {
                    let sp0 = self.bump(TokenKind::KwExpectVersion)?;
                    let s = self.parse_string("expected string after `expect_version`")?;
                    items.push(AstToolItem::ExpectVersion(s));
                    self.end_stmt_line(sp0);
                }
                TokenKind::KwSandbox => {
                    let sp0 = self.bump(TokenKind::KwSandbox)?;
                    let b = self.parse_bool("expected bool after `sandbox`")?;
                    items.push(AstToolItem::Sandbox(b));
                    self.end_stmt_line(sp0);
                }
                TokenKind::KwCapsule => {
                    let sp0 = self.bump(TokenKind::KwCapsule)?;
                    let c = self.parse_ident("expected capsule name after `capsule`")?;
                    items.push(AstToolItem::Capsule(c));
                    self.end_stmt_line(sp0);
                }
                _ => {
                    self.diags.push(Diagnostic::error("invalid tool item").with_span(self.ts.peek().span));
                    self.recover_line();
                }
            }

            self.skip_nl();
        }

        let end = self.end_block("expected `.end` to close tool block")?;
        Some(AstTool { name, items, span: Span::join(start, end) })
    }

    /// bake_block ::= "bake" ident nl { bake_item nl } .end
    fn parse_bake(&mut self) -> Option<AstBake> {
        let start = self.bump(TokenKind::KwBakeBlock)?;
        let name = self.parse_ident("expected bake name")?;
        self.require_nl_or_eof("expected newline after bake header");

        let mut items = Vec::new();
        while !self.at_end_block() && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }

            match self.ts.peek().kind {
                TokenKind::KwIn => {
                    let sp0 = self.bump(TokenKind::KwIn)?;
                    let pname = self.parse_ident("expected port name after `in`")?;
                    self.expect(TokenKind::Colon, "expected `:` after in-port name");
                    let ty = self.parse_type_ref()?;
                    items.push(AstBakeItem::InPort { name: pname, ty, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                TokenKind::KwOut => {
                    let sp0 = self.bump(TokenKind::KwOut)?;
                    let pname = self.parse_ident("expected port name after `out`")?;
                    self.expect(TokenKind::Colon, "expected `:` after out-port name");
                    let ty = self.parse_type_ref()?;
                    items.push(AstBakeItem::OutPort { name: pname, ty, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                TokenKind::KwMake => {
                    let sp0 = self.bump(TokenKind::KwMake)?;
                    let nm = self.parse_ident("expected name after `make`")?;
                    let kind = self.parse_ident("expected make kind (glob|file|text|value)")?;
                    let arg = self.parse_string("expected string arg for make")?;
                    items.push(AstBakeItem::Make { name: nm, kind, arg, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                TokenKind::KwRun => {
                    let rb = self.parse_run_block()?;
                    items.push(AstBakeItem::Run(rb));
                    self.skip_nl();
                }
                TokenKind::KwCache => {
                    let sp0 = self.bump(TokenKind::KwCache)?;
                    let mode = self.parse_ident("expected cache mode (content|mtime|off)")?;
                    items.push(AstBakeItem::Cache { mode, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                TokenKind::KwOutput => {
                    let sp0 = self.bump(TokenKind::KwOutput)?;
                    let port = self.parse_ident("expected port name after `output`")?;
                    // output <ident> at "path"
                    if self.ts.peek().kind == TokenKind::KwAt {
                        self.ts.next();
                    } else {
                        self.diags.push(Diagnostic::error("expected `at` after `output <port>`").with_span(self.ts.peek().span));
                    }
                    let at = self.parse_string("expected string after `at`")?;
                    items.push(AstBakeItem::OutputAt { port, at, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                _ => {
                    self.diags.push(Diagnostic::error("invalid bake item").with_span(self.ts.peek().span));
                    self.recover_line();
                }
            }
        }

        let end = self.end_block("expected `.end` to close bake block")?;
        Some(AstBake { name, items, span: Span::join(start, end) })
    }

    /// run_block ::= "run" "tool" ident nl { run_item nl } .end
    fn parse_run_block(&mut self) -> Option<AstRunBlock> {
        let start = self.bump(TokenKind::KwRun)?;
        // expect `tool`
        if self.ts.peek().kind == TokenKind::KwTool {
            self.ts.next();
        } else if self.ts.peek().kind == TokenKind::Ident {
            // accept ident "tool" if lexer classified differently (safety)
            if self.ts.peek().text.as_deref() == Some("tool") {
                self.ts.next();
            } else {
                self.diags.push(Diagnostic::error("expected `tool` after `run`").with_span(self.ts.peek().span));
            }
        } else {
            self.diags.push(Diagnostic::error("expected `tool` after `run`").with_span(self.ts.peek().span));
        }

        let tool = self.parse_ident("expected tool name after `run tool`")?;
        self.require_nl_or_eof("expected newline after run header");

        let mut items = Vec::new();
        while !self.at_end_block() && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }

            match self.ts.peek().kind {
                TokenKind::KwTakes => {
                    let sp0 = self.bump(TokenKind::KwTakes)?;
                    let port = self.parse_ident("expected port after `takes`")?;
                    // takes <ident> as "<flag>"
                    if self.ts.peek().kind == TokenKind::KwAs {
                        self.ts.next();
                    } else {
                        self.diags.push(Diagnostic::error("expected `as` after `takes <port>`").with_span(self.ts.peek().span));
                    }
                    let flag = self.parse_string("expected string flag after `as`")?;
                    items.push(AstRunItem::Takes { port, flag, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                TokenKind::KwEmits => {
                    let sp0 = self.bump(TokenKind::KwEmits)?;
                    let port = self.parse_ident("expected port after `emits`")?;
                    if self.ts.peek().kind == TokenKind::KwAs {
                        self.ts.next();
                    } else {
                        self.diags.push(Diagnostic::error("expected `as` after `emits <port>`").with_span(self.ts.peek().span));
                    }
                    let flag = self.parse_string("expected string flag after `as`")?;
                    items.push(AstRunItem::Emits { port, flag, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                TokenKind::KwSet => {
                    let sp0 = self.bump(TokenKind::KwSet)?;
                    let flag = self.parse_string("expected string flag after `set`")?;
                    let value = self.parse_value()?;
                    items.push(AstRunItem::Set { flag, value, span: Span::join(sp0, self.prev_span()) });
                    self.skip_nl();
                }
                _ => {
                    self.diags.push(Diagnostic::error("invalid run item").with_span(self.ts.peek().span));
                    self.recover_line();
                }
            }
        }

        let end = self.end_block("expected `.end` to close run block")?;
        Some(AstRunBlock { tool, items, span: Span::join(start, end) })
    }

    /// wire_stmt ::= "wire" ref "->" ref
    fn parse_wire(&mut self) -> Option<AstWire> {
        let start = self.bump(TokenKind::KwWire)?;
        let from = self.parse_ref("expected ref after `wire`")?;
        self.expect(TokenKind::Arrow, "expected `->` in wire");
        let to = self.parse_ref("expected destination ref after `->`")?;
        Some(AstWire { from, to, span: Span::join(start, self.prev_span()) })
    }

    /// export_stmt ::= "export" ref
    fn parse_export(&mut self) -> Option<AstExport> {
        let start = self.bump(TokenKind::KwExport)?;
        let what = self.parse_ref("expected ref after `export`")?;
        Some(AstExport { what, span: Span::join(start, self.prev_span()) })
    }

    /// plan_block ::= "plan" ident nl { "run" ( "exports" | ref ) nl } .end
    fn parse_plan(&mut self) -> Option<AstPlan> {
        let start = self.bump(TokenKind::KwPlan)?;
        let name = self.parse_ident("expected plan name")?;
        self.require_nl_or_eof("expected newline after plan header");

        let mut items = Vec::new();
        while !self.at_end_block() && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }

            // run ...
            let sp0 = self.bump(TokenKind::KwRun)?;
            if self.ts.peek().kind == TokenKind::KwExports {
                let spx = self.ts.next().span;
                items.push(AstPlanItem::RunExports { span: Span::join(sp0, spx) });
                self.skip_nl();
                continue;
            }

            let r = self.parse_ref("expected ref after `run`")?;
            items.push(AstPlanItem::RunRef { what: r, span: Span::join(sp0, self.prev_span()) });
            self.skip_nl();
        }

        let end = self.end_block("expected `.end` to close plan")?;
        Some(AstPlan { name, items, span: Span::join(start, end) })
    }

    /// switch_block ::= "switch" nl { "flag" string_lit action nl } .end
    fn parse_switch(&mut self) -> Option<AstSwitch> {
        let start = self.bump(TokenKind::KwSwitch)?;
        self.require_nl_or_eof("expected newline after `switch`");

        let mut flags = Vec::new();
        while !self.at_end_block() && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }

            let sp0 = self.bump(TokenKind::KwFlag)?;
            let flag = self.parse_string("expected string flag after `flag`")?;
            let action = self.parse_switch_action()?;
            flags.push(AstSwitchFlag { flag, action, span: Span::join(sp0, self.prev_span()) });
            self.skip_nl();
        }

        let end = self.end_block("expected `.end` to close switch")?;
        Some(AstSwitch { flags, span: Span::join(start, end) })
    }

    fn parse_switch_action(&mut self) -> Option<AstSwitchAction> {
        match self.ts.peek().kind {
            TokenKind::KwSet => {
                let sp0 = self.bump(TokenKind::KwSet)?;
                if self.ts.peek().kind == TokenKind::KwPlan {
                    // set plan "name"
                    let sp1 = self.bump(TokenKind::KwPlan)?;
                    let plan = self.parse_string("expected plan name string")?;
                    Some(AstSwitchAction::SetPlan { plan, span: Span::join(sp0, sp1) })
                } else {
                    // set ident value
                    let key = self.parse_ident("expected key after `set`")?;
                    let value = self.parse_value()?;
                    Some(AstSwitchAction::Set { key, value, span: Span::join(sp0, self.prev_span()) })
                }
            }
            TokenKind::KwRun => {
                let sp0 = self.bump(TokenKind::KwRun)?;
                if self.ts.peek().kind == TokenKind::KwExports {
                    let sp1 = self.ts.next().span;
                    Some(AstSwitchAction::RunExports { span: Span::join(sp0, sp1) })
                } else {
                    let r = self.parse_ref("expected ref after `run`")?;
                    Some(AstSwitchAction::RunRef { what: r, span: Span::join(sp0, self.prev_span()) })
                }
            }
            _ => {
                self.diags.push(Diagnostic::error("invalid switch action").with_span(self.ts.peek().span));
                None
            }
        }
    }

    /// ref ::= ident | ident "." ident
    fn parse_ref(&mut self, msg: &str) -> Option<AstRef> {
        let a_tok = self.ts.peek().clone();
        let a = self.parse_ident(msg)?;
        if self.ts.peek().kind == TokenKind::Dot {
            let dot_sp = self.ts.next().span;
            let b = self.parse_ident("expected port name after `.`")?;
            Some(AstRef::BakePort { bake: a, port: b, span: Span::join(a_tok.span, dot_sp) })
        } else {
            Some(AstRef::Var(a))
        }
    }

    /// type_ref ::= prim_type | artifact_type
    fn parse_type_ref(&mut self) -> Option<AstTypeRef> {
        // prim types can be lexed as Ident; we interpret here.
        let first = self.parse_ident("expected type")?;
        let first_s = self.interner.get(first).unwrap_or("");

        let mut path = vec![first];
        while self.ts.peek().kind == TokenKind::Dot {
            self.ts.next();
            let seg = self.parse_ident("expected type segment after `.`")?;
            path.push(seg);
        }

        if path.len() == 1 {
            match first_s {
                "text" => Some(AstTypeRef::Prim(PrimType::Text)),
                "int" => Some(AstTypeRef::Prim(PrimType::Int)),
                "bool" => Some(AstTypeRef::Prim(PrimType::Bool)),
                "bytes" => Some(AstTypeRef::Prim(PrimType::Bytes)),
                _ => Some(AstTypeRef::Artifact(path)),
            }
        } else {
            Some(AstTypeRef::Artifact(path))
        }
    }

    fn parse_value(&mut self) -> Option<AstValue> {
        match self.ts.peek().kind {
            TokenKind::String => {
                let t = self.ts.next().clone();
                let s = t.text.unwrap_or_default();
                let id = self.interner.intern(s);
                Some(AstValue::Str(id))
            }
            TokenKind::Int => {
                let t = self.ts.next().clone();
                let v = t.text.as_deref().and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
                Some(AstValue::Int(v))
            }
            TokenKind::KwTrue => {
                let _ = self.ts.next();
                Some(AstValue::Bool(true))
            }
            TokenKind::KwFalse => {
                let _ = self.ts.next();
                Some(AstValue::Bool(false))
            }
            TokenKind::LBracket => self.parse_list_value(),
            TokenKind::Ident => {
                let id = self.parse_ident("expected identifier value")?;
                Some(AstValue::Ident(id))
            }
            _ => {
                self.diags.push(Diagnostic::error("expected value").with_span(self.ts.peek().span));
                None
            }
        }
    }

    fn parse_list_value(&mut self) -> Option<AstValue> {
        self.expect(TokenKind::LBracket, "expected `[`");
        let mut xs = Vec::new();
        self.skip_nl();

        while self.ts.peek().kind != TokenKind::RBracket && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Comma {
                self.ts.next();
                self.skip_nl();
                continue;
            }
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }
            let v = self.parse_value()?;
            xs.push(v);
            self.skip_nl();
            if self.ts.peek().kind == TokenKind::Comma {
                self.ts.next();
                self.skip_nl();
            }
        }

        self.expect(TokenKind::RBracket, "expected `]` to close list");
        Some(AstValue::List(xs))
    }

    fn parse_list_string(&mut self) -> Option<Vec<NameId>> {
        self.expect(TokenKind::LBracket, "expected `[`");
        let mut xs = Vec::new();
        self.skip_nl();

        while self.ts.peek().kind != TokenKind::RBracket && self.ts.peek().kind != TokenKind::Eof {
            if self.ts.peek().kind == TokenKind::Comma {
                self.ts.next();
                self.skip_nl();
                continue;
            }
            if self.ts.peek().kind == TokenKind::Newline {
                self.skip_nl();
                continue;
            }
            let s = self.parse_string("expected string in list")?;
            xs.push(s);
            self.skip_nl();
            if self.ts.peek().kind == TokenKind::Comma {
                self.ts.next();
                self.skip_nl();
            }
        }

        self.expect(TokenKind::RBracket, "expected `]` to close list");
        Some(xs)
    }

    fn parse_string(&mut self, msg: &str) -> Option<NameId> {
        let t = self.ts.peek().clone();
        if t.kind == TokenKind::String {
            let t = self.ts.next().clone();
            let s = t.text.unwrap_or_default();
            Some(self.interner.intern(s))
        } else {
            self.diags.push(Diagnostic::error(msg).with_span(t.span));
            None
        }
    }

    fn parse_bool(&mut self, msg: &str) -> Option<bool> {
        match self.ts.peek().kind {
            TokenKind::KwTrue => {
                self.ts.next();
                Some(true)
            }
            TokenKind::KwFalse => {
                self.ts.next();
                Some(false)
            }
            _ => {
                self.diags.push(Diagnostic::error(msg).with_span(self.ts.peek().span));
                None
            }
        }
    }

    fn parse_ident(&mut self, msg: &str) -> Option<NameId> {
        let t = self.ts.peek().clone();
        if t.kind == TokenKind::Ident {
            let t = self.ts.next().clone();
            let s = t.text.unwrap_or_default();
            Some(self.interner.intern(s))
        } else {
            self.diags.push(Diagnostic::error(msg).with_span(t.span));
            None
        }
    }

    /// --------------------------------------------------------
    /// Small helpers
    /// --------------------------------------------------------

    fn bump(&mut self, k: TokenKind) -> Option<Span> {
        if self.ts.peek().kind == k {
            Some(self.ts.next().span)
        } else {
            self.diags.push(Diagnostic::error(format!("expected {:?}", k)).with_span(self.ts.peek().span));
            None
        }
    }

    fn expect(&mut self, k: TokenKind, msg: &str) {
        if self.ts.peek().kind == k {
            self.ts.next();
        } else {
            self.diags.push(Diagnostic::error(msg).with_span(self.ts.peek().span));
        }
    }

    fn skip_nl(&mut self) {
        while self.ts.peek().kind == TokenKind::Newline {
            self.ts.next();
        }
    }

    fn require_nl_or_eof(&mut self, msg: &str) {
        match self.ts.peek().kind {
            TokenKind::Newline | TokenKind::Eof => {}
            _ => self.diags.push(Diagnostic::error(msg).with_span(self.ts.peek().span)),
        }
        self.skip_nl();
    }

    fn at_end_block(&self) -> bool {
        self.ts.peek().kind == TokenKind::DotEnd
    }

    fn end_block(&mut self, msg: &str) -> Option<Span> {
        let t = self.ts.peek().clone();
        if t.kind == TokenKind::DotEnd {
            Some(self.ts.next().span)
        } else {
            self.diags.push(Diagnostic::error(msg).with_span(t.span));
            None
        }
    }

    fn prev_span(&self) -> Span {
        // TokenStream ne garde pas last span, donc on approx: peek span.
        // Dans la pratique, un wrapper TrackSpan sera préférable.
        self.ts.peek().span
    }

    fn end_stmt_line(&mut self, _start: Span) {
        // ligne-based : accepter newline ou fin de block
        // (pas d'action, mais utile si on veut des checks stricts)
        let _ = _start;
    }

    fn recover_line(&mut self) {
        while self.ts.peek().kind != TokenKind::Newline
            && self.ts.peek().kind != TokenKind::DotEnd
            && self.ts.peek().kind != TokenKind::Eof
        {
            self.ts.next();
        }
    }

    fn recover_stmt(&mut self) {
        // skip until newline or .end or eof
        self.recover_line();
        self.skip_nl();
        // if we are stuck in a block, do not consume .end here
    }
}

/// ------------------------------------------------------------
/// Optional: AST -> HIR value conversions (helpers)
/// ------------------------------------------------------------

pub fn ast_value_to_hir(v: &AstValue) -> Value {
    match v {
        AstValue::Str(x) => Value::Str(*x),
        AstValue::Int(i) => Value::Int(*i),
        AstValue::Bool(b) => Value::Bool(*b),
        AstValue::Ident(x) => Value::Ident(*x),
        AstValue::List(xs) => Value::List(xs.iter().map(ast_value_to_hir).collect()),
    }
}

pub fn ast_type_to_hir(t: &AstTypeRef) -> TypeRef {
    match t {
        AstTypeRef::Prim(p) => TypeRef::Prim(p.clone()),
        AstTypeRef::Artifact(path) => TypeRef::Artifact(crate::hir::ArtifactType { path: path.clone() }),
    }
}