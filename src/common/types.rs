use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserType {
    CodesdevsSuperadmin,
    CodesdevsSuporte,
    RevendaAdmin,
    RevendaSuporte,
    RevendaSuporteAvancado,
    RevendaFinanceiro,
    RevendaGerente,
    RevendaContador,
    ClienteAdmin,
    ClienteGerente,
    ClienteFuncionario,
    ClienteContador,
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CodesdevsSuperadmin => write!(f, "CODESDEVS_SUPERADMIN"),
            Self::CodesdevsSuporte => write!(f, "CODESDEVS_SUPORTE"),
            Self::RevendaAdmin => write!(f, "REVENDA_ADMIN"),
            Self::RevendaSuporte => write!(f, "REVENDA_SUPORTE"),
            Self::RevendaSuporteAvancado => write!(f, "REVENDA_SUPORTE_AVANCADO"),
            Self::RevendaFinanceiro => write!(f, "REVENDA_FINANCEIRO"),
            Self::RevendaGerente => write!(f, "REVENDA_GERENTE"),
            Self::RevendaContador => write!(f, "REVENDA_CONTADOR"),
            Self::ClienteAdmin => write!(f, "CLIENTE_ADMIN"),
            Self::ClienteGerente => write!(f, "CLIENTE_GERENTE"),
            Self::ClienteFuncionario => write!(f, "CLIENTE_FUNCIONARIO"),
            Self::ClienteContador => write!(f, "CLIENTE_CONTADOR"),
        }
    }
}

impl From<UserType> for crate::entities::sea_orm_active_enums::UserType {
    fn from(ut: UserType) -> Self {
        match ut {
            UserType::CodesdevsSuperadmin => Self::CodesdevsSuperadmin,
            UserType::CodesdevsSuporte => Self::CodesdevsSuporte,
            UserType::RevendaAdmin => Self::RevendaAdmin,
            UserType::RevendaSuporte => Self::RevendaSuporte,
            UserType::RevendaSuporteAvancado => Self::RevendaSuporteAvancado,
            UserType::RevendaFinanceiro => Self::RevendaFinanceiro,
            UserType::RevendaGerente => Self::RevendaGerente,
            UserType::RevendaContador => Self::RevendaContador,
            UserType::ClienteAdmin => Self::ClienteAdmin,
            UserType::ClienteGerente => Self::ClienteGerente,
            UserType::ClienteFuncionario => Self::ClienteFuncionario,
            UserType::ClienteContador => Self::ClienteContador,
        }
    }
}

impl From<crate::entities::sea_orm_active_enums::UserType> for UserType {
    fn from(ut: crate::entities::sea_orm_active_enums::UserType) -> Self {
        match ut {
            crate::entities::sea_orm_active_enums::UserType::CodesdevsSuperadmin => Self::CodesdevsSuperadmin,
            crate::entities::sea_orm_active_enums::UserType::CodesdevsSuporte => Self::CodesdevsSuporte,
            crate::entities::sea_orm_active_enums::UserType::RevendaAdmin => Self::RevendaAdmin,
            crate::entities::sea_orm_active_enums::UserType::RevendaSuporte => Self::RevendaSuporte,
            crate::entities::sea_orm_active_enums::UserType::RevendaSuporteAvancado => Self::RevendaSuporteAvancado,
            crate::entities::sea_orm_active_enums::UserType::RevendaFinanceiro => Self::RevendaFinanceiro,
            crate::entities::sea_orm_active_enums::UserType::RevendaGerente => Self::RevendaGerente,
            crate::entities::sea_orm_active_enums::UserType::RevendaContador => Self::RevendaContador,
            crate::entities::sea_orm_active_enums::UserType::ClienteAdmin => Self::ClienteAdmin,
            crate::entities::sea_orm_active_enums::UserType::ClienteGerente => Self::ClienteGerente,
            crate::entities::sea_orm_active_enums::UserType::ClienteFuncionario => Self::ClienteFuncionario,
            crate::entities::sea_orm_active_enums::UserType::ClienteContador => Self::ClienteContador,
        }
    }
}

impl std::str::FromStr for UserType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CODESDEVS_SUPERADMIN" => Ok(Self::CodesdevsSuperadmin),
            "CODESDEVS_SUPORTE" => Ok(Self::CodesdevsSuporte),
            "REVENDA_ADMIN" => Ok(Self::RevendaAdmin),
            "REVENDA_SUPORTE" => Ok(Self::RevendaSuporte),
            "REVENDA_SUPORTE_AVANCADO" => Ok(Self::RevendaSuporteAvancado),
            "REVENDA_FINANCEIRO" => Ok(Self::RevendaFinanceiro),
            "REVENDA_GERENTE" => Ok(Self::RevendaGerente),
            "REVENDA_CONTADOR" => Ok(Self::RevendaContador),
            "CLIENTE_ADMIN" => Ok(Self::ClienteAdmin),
            "CLIENTE_GERENTE" => Ok(Self::ClienteGerente),
            "CLIENTE_FUNCIONARIO" => Ok(Self::ClienteFuncionario),
            "CLIENTE_CONTADOR" => Ok(Self::ClienteContador),
            _ => Err(format!("Invalid user type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rstest::rstest]
    #[case("CODESDEVS_SUPERADMIN", UserType::CodesdevsSuperadmin)]
    #[case("CODESDEVS_SUPORTE", UserType::CodesdevsSuporte)]
    #[case("REVENDA_ADMIN", UserType::RevendaAdmin)]
    #[case("REVENDA_SUPORTE", UserType::RevendaSuporte)]
    #[case("REVENDA_SUPORTE_AVANCADO", UserType::RevendaSuporteAvancado)]
    #[case("REVENDA_FINANCEIRO", UserType::RevendaFinanceiro)]
    #[case("REVENDA_GERENTE", UserType::RevendaGerente)]
    #[case("REVENDA_CONTADOR", UserType::RevendaContador)]
    #[case("CLIENTE_ADMIN", UserType::ClienteAdmin)]
    #[case("CLIENTE_GERENTE", UserType::ClienteGerente)]
    #[case("CLIENTE_FUNCIONARIO", UserType::ClienteFuncionario)]
    #[case("CLIENTE_CONTADOR", UserType::ClienteContador)]
    fn test_from_str_ok(#[case] input: &str, #[case] expected: UserType) {
        let parsed: UserType = input.parse().unwrap();
        assert_eq!(parsed, expected);
    }

    #[test]
    fn test_from_str_invalid() {
        let result: Result<UserType, String> = "INVALID_TYPE".parse();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid user type"));
    }

    #[test]
    fn test_from_str_empty() {
        let result: Result<UserType, String> = "".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_lowercase() {
        let result: Result<UserType, String> = "codesdevs_superadmin".parse();
        assert!(result.is_err());
    }

    #[rstest::rstest]
    #[case(UserType::CodesdevsSuperadmin, "CODESDEVS_SUPERADMIN")]
    #[case(UserType::CodesdevsSuporte, "CODESDEVS_SUPORTE")]
    #[case(UserType::RevendaAdmin, "REVENDA_ADMIN")]
    #[case(UserType::RevendaSuporte, "REVENDA_SUPORTE")]
    #[case(UserType::RevendaSuporteAvancado, "REVENDA_SUPORTE_AVANCADO")]
    #[case(UserType::RevendaFinanceiro, "REVENDA_FINANCEIRO")]
    #[case(UserType::ClienteAdmin, "CLIENTE_ADMIN")]
    #[case(UserType::ClienteFuncionario, "CLIENTE_FUNCIONARIO")]
    fn test_display(#[case] user_type: UserType, #[case] expected: &str) {
        assert_eq!(user_type.to_string(), expected);
    }

    #[test]
    fn test_serde_round_trip() {
        let variants = [
            UserType::CodesdevsSuperadmin,
            UserType::RevendaSuporteAvancado,
            UserType::ClienteContador,
        ];
        for variant in variants {
            let json = serde_json::to_string(&variant).unwrap();
            let deserialized: UserType = serde_json::from_str(&json).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn test_serde_screaming_snake_case() {
        let json = serde_json::to_value(UserType::CodesdevsSuperadmin).unwrap();
        assert_eq!(json, serde_json::json!("CODESDEVS_SUPERADMIN"));
    }

    #[test]
    fn test_clone_and_partial_eq() {
        let a = UserType::RevendaAdmin;
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_user_type_discriminants() {
        assert_ne!(UserType::ClienteAdmin as u8, UserType::ClienteFuncionario as u8);
    }
}
