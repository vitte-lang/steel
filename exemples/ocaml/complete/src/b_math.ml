(* Small pure helpers to keep the example non-trivial. *)
let rec fib n =
  if n <= 1 then n else fib (n - 1) + fib (n - 2)

let checksum s =
  let acc = ref 0 in
  String.iter (fun c -> acc := (!acc + Char.code c) land 0xFFFF) s;
  !acc
