pub fn namevalid(name: &String) -> bool {
    for ch in name.chars() {
        let c_string = ch.to_string();
        let c: &str = c_string.as_str();

        if !((c >= "a" && c <= "z") || (c >= "0" && c <= "9") || c == "-") {
            return false;
        }
    }
    return true;
}
