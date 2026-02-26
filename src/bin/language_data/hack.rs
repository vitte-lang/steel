use super::super::LangSnippet;

const KEYWORDS: &[&str] = &[
    "<?hh", "abstract", "as", "async", "await", "break", "case", "catch", "class", "const",
    "continue", "default", "do", "else", "enum", "extends", "final", "for", "foreach", "function",
    "if", "implements", "inout", "interface", "namespace", "new", "private", "protected", "public",
    "return", "static", "switch", "throw", "trait", "try", "type", "use", "var", "where", "while",
];

const BUILTINS: &[&str] = &[
    "vec", "dict", "keyset", "shape", "HH\\Lib\\Vec", "HH\\Lib\\Dict", "HH\\Lib\\Str",
    "invariant", "echo", "print", "Exception", "Awaitable", "ResultOrExceptionWrapper",
];

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Hack, trigger: "main", label: "Hack entrypoint", body: "<?hh\n\n<<__EntryPoint>>\nfunction main(): void {\n  ${1:// TODO}\n}" },
    LangSnippet { lang: super::super::Language::Hack, trigger: "func", label: "Hack function", body: "function ${1:name}(${2:mixed} $arg): ${3:void} {\n  ${4:// TODO}\n}" },
    LangSnippet { lang: super::super::Language::Hack, trigger: "test", label: "Hack test", body: "function test_${1:name}(): void {\n  invariant(${2:true}, '${3:failed}');\n}" },
];

pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
