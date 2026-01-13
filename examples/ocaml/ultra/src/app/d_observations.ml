(* Observation model for the astronomy logbook. *)
type entry = {
  id: string;
  object_name: string;
  magnitude: float;
  notes: string;
}

let name e = e.object_name
let ident e = e.id
let mag e = e.magnitude

let sample () = [
  { id = "NGC-7000"; object_name = "North America Nebula"; magnitude = 4.0; notes = "Wide field" };
  { id = "M31"; object_name = "Andromeda Galaxy"; magnitude = 3.4; notes = "Visible to naked eye" };
  { id = "M42"; object_name = "Orion Nebula"; magnitude = 4.0; notes = "Bright core" };
]
