(* Build a simple report from observations. *)
let render entries =
  let lines = List.map (fun e ->
    Printf.sprintf "- %s (%s), mag %.1f"
      (D_observations.name e)
      (D_observations.ident e)
      (D_observations.mag e)
  ) entries in
  String.concat "\n" lines
