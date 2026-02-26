const TYPES: &[&str] = &[
    "char", "short", "int", "long", "float", "double", "void", "signed", "unsigned", "size_t",
    "ssize_t", "intptr_t", "uintptr_t", "int8_t", "int16_t", "int32_t", "int64_t", "uint8_t",
    "uint16_t", "uint32_t", "uint64_t", "bool", "wchar_t",
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::C_KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { super::c_holyc_shared::c_builtins() }
pub(super) fn types() -> &'static [&'static str] { TYPES }
