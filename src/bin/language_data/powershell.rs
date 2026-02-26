use super::super::LangSnippet;

const KEYWORDS: &[&str] = &["function", "filter", "workflow", "if", "else", "elseif", "switch", "for", "foreach", "while", "do", "return", "break", "continue", "param", "class", "enum", "try", "catch", "finally"];
const BUILTINS: &[&str] = &["Write-Host", "Write-Output", "Get-Item", "Get-ChildItem", "Set-Item", "Test-Path", "Where-Object", "ForEach-Object", "Select-Object", "Sort-Object", "Measure-Object", "Out-File"];
const SNIPPETS: &[LangSnippet] = &super::main_func_test_snippets(
    super::super::Language::PowerShell,
    "PowerShell script",
    "param()\nWrite-Host \"${1:hello}\"",
    "PowerShell function",
    "function ${1:Name} {\n  param(${2:[string]$Value})\n  ${3:# TODO}\n}",
    "PowerShell test it",
    "It '${1:works}' {\n  ${2:$true} | Should -Be ${3:$true}\n}",
);
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
pub(super) fn snippets() -> &'static [LangSnippet] { SNIPPETS }
