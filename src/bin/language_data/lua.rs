use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Lua, trigger: "main", label: "Lua script", body: "local function main()\n  print(\"${1:hello}\")\nend\n\nmain()" },
    LangSnippet { lang: super::super::Language::Lua, trigger: "func", label: "Lua function", body: "local function ${1:name}(${2:args})\n  ${3:-- TODO}\nend" },
    LangSnippet { lang: super::super::Language::Lua, trigger: "test", label: "Lua test", body: "local function test_${1:name}()\n  assert(${2:cond})\nend" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::LUA_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::LUA_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
