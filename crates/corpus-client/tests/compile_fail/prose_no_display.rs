// Leak-prevention 1b: `Prose` has no `Display`, so `format!("{}", prose)` /
// `write!(w, "{prose}")` do not compile. Request prose (hostnames, usernames,
// paths) lives in `Prose` leaves, so it cannot reach a print/format sink
// without an explicit, denylisted `.as_str()`/`.into_inner()`.
fn main() {
    let prose = common::Prose::new("secret");
    let _ = format!("{}", prose);
}
