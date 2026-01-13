(* Tiny stats utilities. *)
let mean xs =
  let sum = List.fold_left ( +. ) 0.0 xs in
  sum /. float_of_int (List.length xs)

let checksum s =
  let acc = ref 0 in
  String.iter (fun c -> acc := (!acc + Char.code c) land 0xFFFF) s;
  !acc
