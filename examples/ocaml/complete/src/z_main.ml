let () =
  let title = "ocaml-complete" in
  print_endline (C_util.banner title);
  print_endline (C_util.kv "os" (A_platform.os_name ()));
  print_endline (C_util.kv "user" (A_platform.user ()));
  let fib_n = 10 in
  let fib_v = B_math.fib fib_n in
  print_endline (C_util.kv "fib(10)" (string_of_int fib_v));
  let sum = B_math.checksum title in
  print_endline (C_util.kv "checksum" (string_of_int sum))
