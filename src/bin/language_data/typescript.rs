use super::super::LangSnippet;

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::TypeScript, trigger: "func", label: "TS function", body: "function ${1:name}(${2:args}: ${3:any}): ${4:void} {\n  ${5:// TODO}\n}" },
    LangSnippet { lang: super::super::Language::TypeScript, trigger: "jest", label: "TS Jest describe/it", body: "describe('${1:name}', () => {\n  it('${2:works}', () => {\n    expect(${3:value}).toBe(${4:expected});\n  });\n});" },
    LangSnippet { lang: super::super::Language::TypeScript, trigger: "vitest", label: "TS Vitest test", body: "import { describe, it, expect } from 'vitest';\n\ndescribe('${1:name}', () => {\n  it('${2:works}', () => {\n    expect(${3:value}).toBe(${4:expected});\n  });\n});" },
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::TS_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::super::TS_BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
