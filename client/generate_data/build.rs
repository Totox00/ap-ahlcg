fn main() {
  // recompile when data changes
  println!("cargo:rerun-if-changed=src/data/");
  println!("cargo:rerun-if-changed=src/cards.json");
  println!("cargo:rerun-if-changed=src/prefix.py");
}
