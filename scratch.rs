fn redact_build_host(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars().peekable();
    let mut word = String::new();
    while let Some(c) = chars.next() {
        if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '@' {
            word.push(c);
        } else {
            if word.contains('@') {
                out.push_str("redacted@redacted");
            } else {
                out.push_str(&word);
            }
            word.clear();
            out.push(c);
        }
    }
    if word.contains('@') {
        out.push_str("redacted@redacted");
    } else {
        out.push_str(&word);
    }
    out
}
fn main() {
    println!("{}", redact_build_host("6.9.1-arch1-1 (linux@archlinux)"));
}
