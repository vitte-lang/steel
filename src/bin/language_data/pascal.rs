use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Pascal, trigger: "main", label: "Pascal main", body: "program ${1:Main};\n\nbegin\n  writeln('${2:hello}');\nend." },
    LangSnippet { lang: super::super::Language::Pascal, trigger: "func", label: "Pascal function", body: "function ${1:Name}(${2:Args}): ${3:Integer};\nbegin\n  ${4:// TODO}\nend;" },
    LangSnippet { lang: super::super::Language::Pascal, trigger: "test", label: "Pascal test proc", body: "procedure Test${1:Name};\nbegin\n  ${2:// TODO}\nend;" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::PASCAL_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::PASCAL_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
