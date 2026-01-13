(* Formatting helpers. *)
let line title =
  let bar = String.make (String.length title) '-' in
  title ^ "\n" ^ bar

let kv k v = k ^ ": " ^ v
