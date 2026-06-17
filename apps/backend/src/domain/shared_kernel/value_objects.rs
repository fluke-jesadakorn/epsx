// kernel extraction wave9 — re-export shim
// The 10 value-object files moved into the `epsx-contracts` crate under
// `value_objects`. We re-export the public surface so that
// `use epsx_contracts::value_objects::*` keeps working
// during the wave-10 bulk-rename pass.
pub use epsx_contracts::value_objects::*;
