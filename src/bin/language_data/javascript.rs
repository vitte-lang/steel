use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::JavaScript, trigger: "func", label: "JS function", body: "function ${1:name}(${2:args}) {\n  ${3:// TODO}\n}" },
    LangSnippet { lang: super::super::Language::JavaScript, trigger: "jest", label: "Jest describe/it", body: "describe('${1:name}', () => {\n  it('${2:works}', () => {\n    expect(${3:value}).toBe(${4:expected});\n  });\n});" },
    LangSnippet { lang: super::super::Language::JavaScript, trigger: "vitest", label: "Vitest test", body: "import { describe, it, expect } from 'vitest';\n\ndescribe('${1:name}', () => {\n  it('${2:works}', () => {\n    expect(${3:value}).toBe(${4:expected});\n  });\n});" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::JS_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::JS_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
