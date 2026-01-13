let () =
  let mean = C_stats.mean [1.0; 2.0; 3.0] in
  if abs_float (mean -. 2.0) > 0.0001 then (
    prerr_endline "mean test failed";
    exit 1
  );

  let sum = C_stats.checksum "abc" in
  if sum <= 0 then (
    prerr_endline "checksum test failed";
    exit 1
  );

  print_endline "tests ok"
