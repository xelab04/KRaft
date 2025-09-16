use regex::Regex;

pub async fn validate_tlssan(tlssan: String) -> Result<bool, String> {
    if !tlssan.is_ascii() {
        return Err("Invalid URL".to_string());
    }

    let domain_pattern = r"^([A-Za-z0-9]([A-Za-z0-9-]{0,61}[A-Za-z0-9])?\.)+[A-Za-z]{2,63}$";
    let re = Regex::new(domain_pattern).unwrap();

    if !re.is_match(&tlssan) {
        return Err("Malformed URL".to_string());
    }

    return Ok(true);
}
