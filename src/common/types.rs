use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, utoipa::ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UserType {
    CodesdevsSuperadmin,
    CodesdevsSuporte,
    RevendaAdmin,
    RevendaSuporte,
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
