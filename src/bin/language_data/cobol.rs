const KEYWORDS: &[&str] = &["IDENTIFICATION", "DIVISION", "ENVIRONMENT", "DATA", "PROCEDURE", "SECTION", "WORKING-STORAGE", "PIC", "MOVE", "IF", "ELSE", "PERFORM", "UNTIL", "DISPLAY", "ACCEPT", "STOP", "RUN", "CALL", "COPY", "VALUE"];
const BUILTINS: &[&str] = &["DISPLAY", "ACCEPT", "STRING", "UNSTRING", "INSPECT", "COMPUTE", "OPEN", "CLOSE", "READ", "WRITE", "REWRITE", "DELETE"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
