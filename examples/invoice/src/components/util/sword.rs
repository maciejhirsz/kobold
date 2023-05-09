// Credit: maciejhirsz
pub fn sword(input: &str) -> (&str, &str) {
    let (left, right) = match input.split_once('|') {
        Some(res) => res,
        None => panic!("unable to sword"),
    };

    (left.trim(), right.trim())
}
