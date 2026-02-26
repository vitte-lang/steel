// Shared C/HolyC builtins kept in one place to avoid drift.
const SHARED_MEM_BUILTINS: &[&str] = &["MemCpy", "MemSet"];
const HOLYC_ONLY_BUILTINS: &[&str] = &["Print", "PutS", "Throw", "Catch"];
const C_BUILTINS: &[&str] = SHARED_MEM_BUILTINS;
const HOLYC_BUILTINS: &[&str] = &["Print", "PutS", "MemCpy", "MemSet", "Throw", "Catch"];

pub(super) fn c_builtins() -> &'static [&'static str] {
    C_BUILTINS
}

pub(super) fn holyc_builtins() -> &'static [&'static str] {
    HOLYC_BUILTINS
}

#[allow(dead_code)]
pub(super) fn holyc_only_builtins() -> &'static [&'static str] {
    HOLYC_ONLY_BUILTINS
}
