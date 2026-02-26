use super::super::LangSnippet;

const KEYWORDS: &[&str] = &["class", "enum", "extension", "mixin", "abstract", "implements", "extends", "with", "void", "var", "final", "const", "if", "else", "switch", "for", "while", "return", "import", "library"];
const BUILTINS: &[&str] = &["String", "int", "double", "bool", "List", "Map", "Set", "Future", "Stream", "print", "Object", "dynamic"];
const SNIPPETS: &[LangSnippet] = &super::main_func_test_snippets(
    super::super::Language::Dart,
    "Dart main",
    "void main() {\n  print('${1:hello}');\n}",
    "Dart function",
    "${1:void} ${2:name}(${3:String arg}) {\n  ${4:// TODO}\n}",
    "Dart group/test",
    "group('${1:group}', () {\n  test('${2:name}', () {\n    expect(${3:actual}, equals(${4:expected}));\n  });\n});",
);
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
