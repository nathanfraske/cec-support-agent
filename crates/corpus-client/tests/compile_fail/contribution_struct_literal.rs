// Leak-prevention 1f: `Contribution`'s fields are private and
// `Contribution::new` is the sole constructor, so a struct-literal that would
// bypass the de-id mint does not compile. This is the `contribution-struct-
// literal-bypass` vector — an embedder cannot hand-build a row around the mint.
fn main() {
    let _bypass = corpus_client::Contribution {
        sign_off: corpus_client::SignOff::HumanConfirmed,
        ..unimplemented!()
    };
}
