(* Formatting helpers. *)
let banner title =
  let line = String.make (String.length title) '-' in
  "[" ^ title ^ "]\n" ^ line

let kv k v =
  k ^ ": " ^ v
