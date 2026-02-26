use super::super::LangSnippet;

const KEYWORDS: &[&str] = &["class", "struct", "enum", "protocol", "func", "let", "var", "if", "else", "switch", "case", "for", "while", "return", "import", "extension", "guard", "defer", "async", "await"];
const BUILTINS: &[&str] = &["String", "Int", "Double", "Float", "Bool", "Array", "Dictionary", "Set", "print", "Result", "Optional", "Any"];
const SNIPPETS: &[LangSnippet] = &super::main_func_test_snippets(
    super::super::Language::Swift,
    "Swift main",
    "@main\nstruct ${1:App} {\n  static func main() {\n    print(\"${2:hello}\")\n  }\n}",
    "Swift function",
    "func ${1:name}(${2:arg}: ${3:String}) -> ${4:Void} {\n  ${5:// TODO}\n}",
    "Swift XCTestCase",
    "import XCTest\n\nfinal class ${1:FeatureTests}: XCTestCase {\n  func test_${2:name}() {\n    XCTAssertEqual(${3:expected}, ${4:actual})\n  }\n}",
);
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
