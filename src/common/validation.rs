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
