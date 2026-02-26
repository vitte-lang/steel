use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Zig, trigger: "main", label: "Zig main", body: "const std = @import(\"std\");\n\npub fn main() !void {\n  std.debug.print(\"${1:hello}\\n\", .{});\n}" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::ZIG_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::ZIG_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
