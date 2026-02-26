use super::super::LangSnippet;

const KEYWORDS: &[&str] = &[
    "alias", "break", "case", "const", "const_assert", "continue", "continuing", "default",
    "diagnostic", "discard", "else", "enable", "false", "fn", "for", "if", "let", "loop",
    "override", "requires", "return", "struct", "switch", "true", "var", "while",
];

const BUILTINS: &[&str] = &[
    "vec2", "vec3", "vec4", "mat2x2", "mat3x3", "mat4x4", "f32", "i32", "u32", "bool",
    "textureSample", "textureLoad", "dot", "cross", "normalize", "length", "clamp", "mix",
];

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Wgsl, trigger: "main", label: "WGSL compute main", body: "@compute @workgroup_size(1)\nfn main(@builtin(global_invocation_id) gid: vec3<u32>) {\n  let _id = gid.x;\n}" },
    LangSnippet { lang: super::super::Language::Wgsl, trigger: "func", label: "WGSL function", body: "fn ${1:name}(${2:x}: f32) -> f32 {\n  return ${3:x};\n}" },
    LangSnippet { lang: super::super::Language::Wgsl, trigger: "test", label: "WGSL test helper", body: "fn test_${1:name}() -> bool {\n  return ${2:true};\n}" },
];

pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
