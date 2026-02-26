const TYPES: &[&str] = &[
    "bool", "char", "char16_t", "char32_t", "wchar_t", "short", "int", "long", "float", "double",
    "void", "size_t", "ssize_t", "intptr_t", "uintptr_t", "int8_t", "int16_t", "int32_t", "int64_t",
    "uint8_t", "uint16_t", "uint32_t", "uint64_t", "string", "u8string", "u16string", "u32string",
    "wstring",
];

pub(super) fn keywords() -> &'static [&'static str] { super::super::CPP_KEYWORDS }
pub(super) fn types() -> &'static [&'static str] { TYPES }
