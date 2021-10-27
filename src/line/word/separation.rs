
use super::Separator;

pub fn get_separator(text: &str) -> (String, Separator) {
    if text.ends_with("L") {
        let mut text = text.to_string();
        text.pop();

        (text, Separator::Line)
    } else {
        (text.to_string(), Separator::Whitespace)
    }
}
