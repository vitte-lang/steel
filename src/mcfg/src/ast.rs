//! AST (Abstract Syntax Tree) for Muffin Bakefile v2.
//!
//! This models the language described by `muffin.ebnf` (Bakefile v2):
//! - top-level blocks end with `.end`
//! - statements: store/capsule/var/profile/tool/bake/wire/export/plan/switch/set
//! - values: string/int/bool/list/ident
//! - refs: `ident` or `bake.port`
//!
//! Notes:
//! - Parsing/tokenization is intentionally out-of-scope here.
//! - Spans are included for diagnostics and tooling.
//! - The AST is designed to be stable, serializable-friendly, and visitor-walkable.

use std::fmt;

/// Byte span in a source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Span {
    pub file_id: u32,
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline]
    pub fn new(file_id: u32, start: u32, end: u32) -> Self {
        Self { file_id, start, end }
    }

    #[inline]
    pub fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn merge(a: Span, b: Span) -> Span {
        if a.file_id != b.file_id {
            // Keep "a" as anchor; callers should avoid cross-file merges.
            return a;
        }
        Span {
            file_id: a.file_id,
            start: a.start.min(b.start),
            end: a.end.max(b.end),
        }
    }
}

/// A value paired with a span.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl<T> Spanned<T> {
    #[inline]
    pub fn new(span: Span, value: T) -> Self {
        Self { span, value }
    }

    #[inline]
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            span: self.span,
            value: f(self.value),
        }
    }
}

/// Identifier.
pub type Ident = Spanned<String>;

/// Integer literal.
pub type IntLit = Spanned<i64>;

/// Boolean literal.
pub type BoolLit = Spanned<bool>;

/// String literal (already unescaped by the parser, ideally).
pub type StringLit = Spanned<String>;

/// Muffin file root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuffinFile {
    pub span: Span,
    pub header: Header,
    pub stmts: Vec<Stmt>,
}

/// `muffin bake <int>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub span: Span,
    pub keyword_muffin: Span,
    pub keyword_bake: Span,
    pub version: IntLit,
}

/// Top-level statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stmt {
    pub span: Span,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StmtKind {
    Store(StoreBlock),
    Capsule(CapsuleBlock),
    Var(VarDecl),
    Profile(ProfileBlock),
    Tool(ToolBlock),
    Bake(BakeBlock),
    Wire(WireStmt),
    Export(ExportStmt),
    Plan(PlanBlock),
    Switch(SwitchBlock),
    Set(SetStmt), // global set
}

/// `set <ident> <value>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetStmt {
    pub span: Span,
    pub key: Ident,
    pub value: Value,
}

/// `store <name> ... .end`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreBlock {
    pub span: Span,
    pub name: Ident,
    pub items: Vec<StoreItem>,
    pub end_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreItem {
    pub span: Span,
    pub kind: StoreItemKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreItemKind {
    Path(StringLit),
    Mode(StoreMode),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreMode {
    pub span: Span,
    pub kind: StoreModeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreModeKind {
    Content,
    Mtime,
    Off,
}

/// `capsule <name> ... .end`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapsuleBlock {
    pub span: Span,
    pub name: Ident,
    pub items: Vec<CapsuleItem>,
    pub end_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapsuleItem {
    pub span: Span,
    pub kind: CapsuleItemKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapsuleItemKind {
    Env(EnvPolicy),
    Fs(FsPolicy),
    Net(NetPolicy),
    Time(TimePolicy),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvPolicy {
    pub span: Span,
    pub kind: EnvPolicyKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvPolicyKind {
    Allow(Vec<StringLit>),
    Deny(Vec<StringLit>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FsPolicy {
    pub span: Span,
    pub kind: FsPolicyKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsPolicyKind {
    AllowRead(Vec<StringLit>),
    AllowWrite(Vec<StringLit>),
    AllowWriteExact(Vec<StringLit>),
    Deny(Vec<StringLit>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetPolicy {
    pub span: Span,
    pub kind: NetPolicyKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetPolicyKind {
    Allow,
    Deny,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimePolicy {
    pub span: Span,
    pub stable: BoolLit,
}

/// `var <name> : <type> = <value>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarDecl {
    pub span: Span,
    pub name: Ident,
    pub ty: TypeRef,
    pub value: Value,
}

/// `text | int | bool | bytes | artifact.path.like`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeRef {
    pub span: Span,
    pub kind: TypeRefKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRefKind {
    Prim(PrimType),
    Artifact(ArtifactType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimType {
    pub span: Span,
    pub kind: PrimTypeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimTypeKind {
    Text,
    Int,
    Bool,
    Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactType {
    pub span: Span,
    /// `ident("." ident)+`
    pub segments: Vec<Ident>,
}

/// `profile <name> ... .end`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileBlock {
    pub span: Span,
    pub name: Ident,
    pub items: Vec<ProfileItem>,
    pub end_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileItem {
    pub span: Span,
    pub key: Ident,
    pub value: Value,
}

/// `tool <name> ... .end`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolBlock {
    pub span: Span,
    pub name: Ident,
    pub items: Vec<ToolItem>,
    pub end_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolItem {
    pub span: Span,
    pub kind: ToolItemKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolItemKind {
    Exec(StringLit),
    ExpectVersion(StringLit),
    Sandbox(BoolLit),
    Capsule(Ident),
}

/// `bake <name> ... .end`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BakeBlock {
    pub span: Span,
    pub name: Ident,
    pub items: Vec<BakeItem>,
    pub end_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BakeItem {
    pub span: Span,
    pub kind: BakeItemKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BakeItemKind {
    In(PortDecl),
    Out(PortDecl),
    Make(MakeStmt),
    Run(RunBlock),
    Cache(CacheStmt),
    Output(OutputStmt),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortDecl {
    pub span: Span,
    pub name: Ident,
    pub ty: TypeRef,
}

/// `make <ident> <glob|file|text|value> "<string>"`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MakeStmt {
    pub span: Span,
    pub name: Ident,
    pub kind: MakeKind,
    pub spec: StringLit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MakeKind {
    pub span: Span,
    pub kind: MakeKindKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MakeKindKind {
    Glob,
    File,
    Text,
    Value,
}

/// `run tool <name> ... .end`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunBlock {
    pub span: Span,
    pub tool: Ident,
    pub items: Vec<RunItem>,
    pub end_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunItem {
    pub span: Span,
    pub kind: RunItemKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunItemKind {
    Takes(TakesStmt),
    Emits(EmitsStmt),
    Set(RunSetStmt),
}

/// `takes <ident> as "<flag>"`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TakesStmt {
    pub span: Span,
    pub port: Ident,
    pub flag: StringLit,
}

/// `emits <ident> as "<flag>"`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitsStmt {
    pub span: Span,
    pub port: Ident,
    pub flag: StringLit,
}

/// `set "<flag>" <value>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSetStmt {
    pub span: Span,
    pub flag: StringLit,
    pub value: Value,
}

/// `cache <content|mtime|off>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStmt {
    pub span: Span,
    pub mode: CacheMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheMode {
    pub span: Span,
    pub kind: CacheModeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheModeKind {
    Content,
    Mtime,
    Off,
}

/// `output <ident> at "<path>"`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputStmt {
    pub span: Span,
    pub port: Ident,
    pub path: StringLit,
}

/// `wire <ref> -> <ref>`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireStmt {
    pub span: Span,
    pub from: Ref,
    pub to: Ref,
}

/// `export <ref>` (must reference an out port)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportStmt {
    pub span: Span,
    pub what: Ref,
}

/// `plan <name> ... .end`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanBlock {
    pub span: Span,
    pub name: Ident,
    pub items: Vec<PlanItem>,
    pub end_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanItem {
    pub span: Span,
    pub run: PlanRun,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanRun {
    pub span: Span,
    pub target: PlanTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanTarget {
    Exports(Span),
    Ref(Ref),
}

/// `switch ... .end`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchBlock {
    pub span: Span,
    pub items: Vec<SwitchItem>,
    pub end_span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchItem {
    pub span: Span,
    pub flag: StringLit,
    pub action: SwitchAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitchAction {
    SetVar { span: Span, key: Ident, value: Value },
    SetPlan { span: Span, plan: StringLit },
    Run { span: Span, target: PlanTarget },
}

/// Reference: `ident` (global var) or `bake.port`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ref {
    pub span: Span,
    pub kind: RefKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefKind {
    Var(Ident),
    Port { bake: Ident, port: Ident },
}

/// Values: string/int/bool/list/ident
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Value {
    pub span: Span,
    pub kind: ValueKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueKind {
    String(StringLit),
    Int(IntLit),
    Bool(BoolLit),
    List(Vec<Value>),
    Ident(Ident),
}

impl Value {
    pub fn as_ident(&self) -> Option<&Ident> {
        match &self.kind {
            ValueKind::Ident(i) => Some(i),
            _ => None,
        }
    }
}

/// -----------------------------------------
/// Visitor / walker (tooling-friendly)
/// -----------------------------------------

pub trait Visitor {
    fn visit_file(&mut self, node: &MuffinFile) {
        walk_file(self, node);
    }
    fn visit_stmt(&mut self, node: &Stmt) {
        walk_stmt(self, node);
    }
    fn visit_value(&mut self, node: &Value) {
        walk_value(self, node);
    }
    fn visit_ref(&mut self, node: &Ref) {
        walk_ref(self, node);
    }
    fn visit_type(&mut self, node: &TypeRef) {
        walk_type(self, node);
    }
}

pub fn walk_file<V: Visitor + ?Sized>(v: &mut V, f: &MuffinFile) {
    for s in &f.stmts {
        v.visit_stmt(s);
    }
}

pub fn walk_stmt<V: Visitor + ?Sized>(v: &mut V, s: &Stmt) {
    match &s.kind {
        StmtKind::Store(b) => {
            for it in &b.items {
                match &it.kind {
                    StoreItemKind::Path(_) => {}
                    StoreItemKind::Mode(_) => {}
                }
            }
        }
        StmtKind::Capsule(b) => {
            for it in &b.items {
                match &it.kind {
                    CapsuleItemKind::Env(p) => match &p.kind {
                        EnvPolicyKind::Allow(xs) | EnvPolicyKind::Deny(xs) => {
                            let _ = xs;
                        }
                    },
                    CapsuleItemKind::Fs(p) => match &p.kind {
                        FsPolicyKind::AllowRead(xs)
                        | FsPolicyKind::AllowWrite(xs)
                        | FsPolicyKind::AllowWriteExact(xs)
                        | FsPolicyKind::Deny(xs) => {
                            let _ = xs;
                        }
                    },
                    CapsuleItemKind::Net(_) => {}
                    CapsuleItemKind::Time(t) => {
                        let _ = t;
                    }
                }
            }
        }
        StmtKind::Var(d) => {
            v.visit_type(&d.ty);
            v.visit_value(&d.value);
        }
        StmtKind::Profile(b) => {
            for it in &b.items {
                v.visit_value(&it.value);
            }
        }
        StmtKind::Tool(b) => {
            for it in &b.items {
                match &it.kind {
                    ToolItemKind::Exec(_) => {}
                    ToolItemKind::ExpectVersion(_) => {}
                    ToolItemKind::Sandbox(_) => {}
                    ToolItemKind::Capsule(_) => {}
                }
            }
        }
        StmtKind::Bake(b) => {
            for it in &b.items {
                match &it.kind {
                    BakeItemKind::In(p) | BakeItemKind::Out(p) => v.visit_type(&p.ty),
                    BakeItemKind::Make(_) => {}
                    BakeItemKind::Run(r) => {
                        for ri in &r.items {
                            match &ri.kind {
                                RunItemKind::Takes(_) => {}
                                RunItemKind::Emits(_) => {}
                                RunItemKind::Set(s2) => v.visit_value(&s2.value),
                            }
                        }
                    }
                    BakeItemKind::Cache(_) => {}
                    BakeItemKind::Output(_) => {}
                }
            }
        }
        StmtKind::Wire(w) => {
            v.visit_ref(&w.from);
            v.visit_ref(&w.to);
        }
        StmtKind::Export(e) => v.visit_ref(&e.what),
        StmtKind::Plan(p) => {
            for it in &p.items {
                match &it.run.target {
                    PlanTarget::Exports(_) => {}
                    PlanTarget::Ref(r) => v.visit_ref(r),
                }
            }
        }
        StmtKind::Switch(sw) => {
            for it in &sw.items {
                match &it.action {
                    SwitchAction::SetVar { value, .. } => v.visit_value(value),
                    SwitchAction::SetPlan { .. } => {}
                    SwitchAction::Run { target, .. } => match target {
                        PlanTarget::Exports(_) => {}
                        PlanTarget::Ref(r) => v.visit_ref(r),
                    },
                }
            }
        }
        StmtKind::Set(ss) => v.visit_value(&ss.value),
    }
}

pub fn walk_value<V: Visitor + ?Sized>(v: &mut V, val: &Value) {
    match &val.kind {
        ValueKind::String(_) | ValueKind::Int(_) | ValueKind::Bool(_) | ValueKind::Ident(_) => {}
        ValueKind::List(xs) => {
            for x in xs {
                v.visit_value(x);
            }
        }
    }
}

pub fn walk_ref<V: Visitor + ?Sized>(_v: &mut V, _r: &Ref) {}

pub fn walk_type<V: Visitor + ?Sized>(_v: &mut V, _t: &TypeRef) {}

/// -----------------------------------------
/// Minimal formatting helpers (debug tooling)
/// -----------------------------------------

impl fmt::Display for Ref {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            RefKind::Var(id) => write!(f, "{}", id.value),
            RefKind::Port { bake, port } => write!(f, "{}.{}", bake.value, port.value),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ValueKind::String(s) => write!(f, "{:?}", s.value),
            ValueKind::Int(i) => write!(f, "{}", i.value),
            ValueKind::Bool(b) => write!(f, "{}", b.value),
            ValueKind::Ident(id) => write!(f, "{}", id.value),
            ValueKind::List(xs) => {
                write!(f, "[")?;
                for (i, x) in xs.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{x}")?;
                }
                write!(f, "]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sp<T>(value: T) -> Spanned<T> {
        Spanned::new(Span::new(0, 0, 0), value)
    }

    #[test]
    fn ref_display_var() {
        let r = Ref {
            span: Span::default(),
            kind: RefKind::Var(sp("x".to_string())),
        };
        assert_eq!(r.to_string(), "x");
    }

    #[test]
    fn ref_display_port() {
        let r = Ref {
            span: Span::default(),
            kind: RefKind::Port {
                bake: sp("app".to_string()),
                port: sp("exe".to_string()),
            },
        };
        assert_eq!(r.to_string(), "app.exe");
    }

    #[test]
    fn value_display_list() {
        let v = Value {
            span: Span::default(),
            kind: ValueKind::List(vec![
                Value { span: Span::default(), kind: ValueKind::Int(sp(1)) },
                Value { span: Span::default(), kind: ValueKind::Bool(sp(true)) },
                Value { span: Span::default(), kind: ValueKind::Ident(sp("k".to_string())) },
            ]),
        };
        assert_eq!(v.to_string(), "[1, true, k]");
    }
}