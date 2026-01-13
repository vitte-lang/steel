let () =
  let title = "Astronomy Logbook" in
  print_endline (B_format.line title);
  print_endline (B_format.kv "generated" (A_time.now_string ()));

  let entries = D_observations.sample () in
  let mags = List.map D_observations.mag entries in
  let mean_mag = C_stats.mean mags in
  print_endline (B_format.kv "avg magnitude" (Printf.sprintf "%.2f" mean_mag));

  let report = E_report.render entries in
  print_endline "";
  print_endline report;

  let sum = C_stats.checksum title in
  print_endline "";
  print_endline (B_format.kv "checksum" (string_of_int sum));
