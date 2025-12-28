pub fn namevalid(name: &String) -> bool {

    return name.chars().all(|ch|
        (ch >= 'a' && ch <= 'z')
        || (ch >= '0' && ch <= '9')
        || ch == '-'
    );

    // for ch in name.chars() {

    //     if (ch.is_ascii_alphanumeric() && ch.is_ascii_lowercase()) || ch == '-' {
    //         continue;
    //     }

    //     // let c_string = ch.to_string();
    //     // let c: &str = c_string.as_str();

    //     // if !((ch >= 'a' && c <= "z") || (c >= "0" && c <= "9") || c == "-") {
    //     //     return false;
    //     // }
    // }
    // return true;
}
