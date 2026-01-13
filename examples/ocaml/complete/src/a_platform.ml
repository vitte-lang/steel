(* Minimal platform helpers for demo output. *)
let os_name () =
  Sys.os_type

let user () =
  try Sys.getenv "USER" with Not_found -> "unknown"
