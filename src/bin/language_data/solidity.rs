use super::super::LangSnippet;

const KEYWORDS: &[&str] = &["contract", "interface", "library", "function", "modifier", "event", "error", "struct", "enum", "mapping", "if", "else", "for", "while", "return", "import", "pragma", "public", "private", "external"];
const BUILTINS: &[&str] = &["address", "uint", "uint256", "int", "bytes", "string", "bool", "msg", "tx", "block", "abi", "keccak256"];
const SNIPPETS: &[LangSnippet] = &super::main_func_test_snippets(
    super::super::Language::Solidity,
    "Solidity contract",
    "pragma solidity ^0.8.0;\n\ncontract ${1:Main} {\n  constructor() {}\n}",
    "Solidity function",
    "function ${1:name}(${2:uint256 x}) public returns (${3:uint256}) {\n  return ${4:x};\n}",
    "Solidity Foundry test",
    "function test_${1:name}() public {\n  assertEq(${2:left}, ${3:right});\n}",
);
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
