const KEYWORDS: &[&str] = &["module", "endmodule", "input", "output", "inout", "wire", "reg", "logic", "always", "always_ff", "always_comb", "if", "else", "case", "for", "generate", "assign", "initial", "parameter", "localparam"];
const BUILTINS: &[&str] = &["$display", "$monitor", "$finish", "$stop", "$time", "$random", "posedge", "negedge", "bit", "byte", "int", "longint"];
pub(super) fn keywords() -> &'static [&'static str] { KEYWORDS }
pub(super) fn builtins() -> &'static [&'static str] { BUILTINS }
