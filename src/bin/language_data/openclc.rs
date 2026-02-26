use super::super::LangSnippet;

const KEYWORDS: &[&str] = &[
    "__kernel", "__global", "__local", "__private", "__constant", "kernel", "global", "local",
    "private", "constant", "for", "while", "if", "else", "return", "typedef", "struct", "union",
    "enum", "inline", "static", "volatile", "const",
];

const BUILTINS: &[&str] = &[
    "get_global_id", "get_local_id", "get_group_id", "get_global_size", "barrier", "mem_fence",
    "read_imagef", "write_imagef", "sampler_t", "float4", "int2", "uchar", "mad", "clamp",
];

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::OpenClC, trigger: "main", label: "OpenCL kernel", body: "__kernel void main_kernel(__global float* out) {\n  int gid = get_global_id(0);\n  out[gid] = 0.0f;\n}" },
    LangSnippet { lang: super::super::Language::OpenClC, trigger: "func", label: "OpenCL helper", body: "inline float ${1:name}(float ${2:x}) {\n  return ${3:x};\n}" },
    LangSnippet { lang: super::super::Language::OpenClC, trigger: "test", label: "OpenCL test helper", body: "inline int test_${1:name}(int ${2:x}) {\n  return ${3:x};\n}" },
];

pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
