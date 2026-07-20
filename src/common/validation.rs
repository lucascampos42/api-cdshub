use crate::errors::AppError;

pub fn validate_email(email: &str) -> Result<(), AppError> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return Err(AppError::bad_request("Email cannot be empty"));
    }
    if trimmed.len() > 254 {
        return Err(AppError::bad_request("Email too long"));
    }
    let at_pos = trimmed.find('@');
    let dot_pos = trimmed.rfind('.');
    match (at_pos, dot_pos) {
        (Some(at), Some(dot)) if at > 0 && dot > at + 1 && dot < trimmed.len() - 1 => Ok(()),
        _ => Err(AppError::bad_request("Invalid email format")),
    }
}

pub fn validate_cpf(cpf: &str) -> Result<(), AppError> {
    let digits: Vec<u8> = cpf
        .chars()
        .filter(|c| c.is_ascii_digit())
        .map(|c| c.to_digit(10).unwrap() as u8)
        .collect();

    if digits.len() != 11 {
        return Err(AppError::bad_request("CPF must have 11 digits"));
    }

    if digits.iter().all(|&d| d == digits[0]) {
        return Err(AppError::bad_request("Invalid CPF"));
    }

    let mut sum = 0u32;
    for i in 0..9 {
        sum += digits[i] as u32 * (10 - i) as u32;
    }
    let rem = sum % 11;
    let dig1 = if rem < 2 { 0 } else { 11 - rem as u8 };
    if digits[9] != dig1 {
        return Err(AppError::bad_request("Invalid CPF"));
    }

    sum = 0;
    for i in 0..10 {
        sum += digits[i] as u32 * (11 - i) as u32;
    }
    let rem = sum % 11;
    let dig2 = if rem < 2 { 0 } else { 11 - rem as u8 };
    if digits[10] != dig2 {
        return Err(AppError::bad_request("Invalid CPF"));
    }

    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.len() < 8 {
        return Err(AppError::bad_request(
            "Senha deve ter no mínimo 8 caracteres",
        ));
    }
    if password.len() > 128 {
        return Err(AppError::bad_request("Senha muito longa (máximo 128 caracteres)"));
    }

    let has_letter = password.chars().any(|c| c.is_ascii_alphabetic());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());

    if !has_letter {
        return Err(AppError::bad_request(
            "Senha deve conter ao menos uma letra",
        ));
    }
    if !has_digit {
        return Err(AppError::bad_request(
            "Senha deve conter ao menos um número",
        ));
    }

    Ok(())
}

pub fn validate_name(name: &str) -> Result<(), AppError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AppError::bad_request("Name cannot be empty"));
    }
    if trimmed.len() < 2 {
        return Err(AppError::bad_request("Name must have at least 2 characters"));
    }
    if trimmed.len() > 255 {
        return Err(AppError::bad_request("Name too long (max 255 characters)"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case("user@example.com")]
    #[case("a@b.co")]
    #[case("test.name+tag@domain.org")]
    #[case("x@y.z")]
    fn test_validate_email_ok(#[case] email: &str) {
        assert!(validate_email(email).is_ok());
    }

    #[rstest::rstest]
    #[case("")]
    #[case("not-an-email")]
    #[case("@domain.com")]
    #[case("user@")]
    #[case("user@.com")]
    #[case("user@domain")]
    #[case("a@b.")]
    fn test_validate_email_invalid(#[case] email: &str) {
        assert!(validate_email(email).is_err());
    }

    #[test]
    fn test_validate_email_too_long() {
        let long = format!("{}@example.com", "a".repeat(245));
        assert!(validate_email(&long).is_err());
    }

    #[test]
    fn test_validate_email_whitespace() {
        assert!(validate_email(" user@example.com").is_ok());
    }

    #[rstest::rstest]
    #[case("529.982.247-25")]
    #[case("52998224725")]
    fn test_validate_cpf_ok(#[case] cpf: &str) {
        assert!(validate_cpf(cpf).is_ok());
    }

    #[rstest::rstest]
    #[case("")]
    #[case("123.456.789-00")]
    #[case("000.000.000-00")]
    #[case("111.111.111-11")]
    #[case("123")]
    #[case("abcdefghijk")]
    fn test_validate_cpf_invalid(#[case] cpf: &str) {
        assert!(validate_cpf(cpf).is_err());
    }

    #[test]
    fn test_validate_cpf_wrong_digit() {
        assert!(validate_cpf("529.982.247-26").is_err());
    }

    #[rstest::rstest]
    #[case("Abc12345")]
    #[case("senha123")]
    #[case("MyP@ssw0rd")]
    #[case("a1b2c3d4e5")]
    fn test_validate_password_ok(#[case] password: &str) {
        assert!(validate_password(password).is_ok());
    }

    #[rstest::rstest]
    #[case("")]
    #[case("abc")]
    #[case("1234567")]
    #[case("abcdefg")]
    #[case("12345678")]
    #[case("ABCDEFGH")]
    fn test_validate_password_too_short_or_no_mix(#[case] password: &str) {
        assert!(validate_password(password).is_err());
    }

    #[test]
    fn test_validate_password_too_long() {
        let long = format!("Ab1{}", "a".repeat(126));
        assert!(validate_password(&long).is_err());
    }

    #[test]
    fn test_validate_password_edge_8_chars() {
        assert!(validate_password("Abcdef1!").is_ok());
    }

    #[rstest::rstest]
    #[case("John")]
    #[case("Maria Silva")]
    #[case("João")]
    #[case("A B")]
    fn test_validate_name_ok(#[case] name: &str) {
        assert!(validate_name(name).is_ok());
    }

    #[rstest::rstest]
    #[case("")]
    #[case("  ")]
    #[case("A")]
    fn test_validate_name_invalid(#[case] name: &str) {
        assert!(validate_name(name).is_err());
    }
}
