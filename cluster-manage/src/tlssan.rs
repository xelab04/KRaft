use regex::Regex;

pub async fn validate_tlssan(tlssan: String) -> Result<bool, String> {
    if !tlssan.is_ascii() {
        return Err("Invalid URL".to_string());
    }

    let domain_pattern = r"^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*$";
    let re = Regex::new(domain_pattern).unwrap();

    if !re.is_match(&tlssan) {
        return Err("Malformed URL".to_string());
    }

    return Ok(true);
}
