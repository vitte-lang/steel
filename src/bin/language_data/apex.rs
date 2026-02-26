use super::super::LangSnippet;

const KEYWORDS: &[&str] = &[
    "abstract", "after", "before", "break", "case", "catch", "class", "const", "continue", "delete",
    "do", "else", "enum", "extends", "final", "finally", "for", "if", "implements", "insert", "interface",
    "new", "private", "protected", "public", "return", "static", "switch", "testmethod", "this", "throw",
    "trigger", "try", "update", "upsert", "virtual", "void", "while", "with", "without",
];

const BUILTINS: &[&str] = &[
    "System", "String", "Integer", "Boolean", "Date", "Datetime", "List", "Map", "Set", "Database",
    "Test", "Limits", "Schema", "Trigger", "UserInfo", "JSON", "Math",
];

const SNIPPETS: &[LangSnippet] = &[
    LangSnippet { lang: super::super::Language::Apex, trigger: "main", label: "Apex class", body: "public with sharing class ${1:Main} {\n  public static void mainMethod() {\n    ${2:// TODO}\n  }\n}" },
    LangSnippet { lang: super::super::Language::Apex, trigger: "func", label: "Apex method", body: "public static ${1:void} ${2:name}(${3:String arg}) {\n  ${4:// TODO}\n}" },
    LangSnippet { lang: super::super::Language::Apex, trigger: "test", label: "Apex test", body: "@IsTest\nprivate class ${1:MainTest} {\n  @IsTest static void test_${2:name}() {\n    Test.startTest();\n    ${3:// TODO}\n    Test.stopTest();\n  }\n}" },
];

pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
