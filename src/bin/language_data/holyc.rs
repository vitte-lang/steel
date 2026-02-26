use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    super::snippet(
        super::super::Language::HolyC,
        "main",
        "HolyC main",
        "U0 Main() {\n  Print(\"${1:hello}\\n\");\n}",
    ),
    super::snippet(
        super::super::Language::HolyC,
        "func",
        "HolyC function",
        "U0 ${1:Name}(${2:args}) {\n  ${3:// TODO}\n}",
    ),
    super::snippet(
        super::super::Language::HolyC,
        "test",
        "HolyC test",
        "U0 Test_${1:Name}() {\n  // type 'assert' for quick checks\n  ${2:// TODO}\n}",
    ),
    super::snippet(
        super::super::Language::HolyC,
        "assert",
        "HolyC assert",
        "if (!(${1:cond})) {\n  Throw(\"${2:assert failed}\");\n}",
    ),
    super::snippet(
        super::super::Language::HolyC,
        "const",
        "HolyC const str",
        "I8 * const ${1:name} = \"${2:value}\";",
    ),
    super::snippet(
        super::super::Language::HolyC,
        "static",
        "HolyC static arr",
        "static I32 ${1:name}[${2:len}] = {${3:0}};",
    ),
    super::snippet(
        super::super::Language::HolyC,
        "struct",
        "HolyC struct",
        "class ${1:Name} {\n  I32 ${2:field};\n};",
    ),
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::HOLYC_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::c_holyc_shared::holyc_builtins() }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
