use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SystemInfo {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub active: bool,
}

pub fn get_all_systems() -> Vec<SystemInfo> {
    vec![
        SystemInfo {
            id: "cds-gestor".to_string(),
            slug: "cds-gestor".to_string(),
            name: "CDS Gestor".to_string(),
            description: "Sistema completo de gestão empresarial (ERP).".to_string(),
            active: true,
        },
        SystemInfo {
            id: "agenda".to_string(),
            slug: "agenda".to_string(),
            name: "Agenda".to_string(),
            description: "Sistema de agendamento e controle de visitas técnicas.".to_string(),
            active: true,
        },
        SystemInfo {
            id: "calculadora-xml".to_string(),
            slug: "calculadora-xml".to_string(),
            name: "Calculadora XML".to_string(),
            description: "Ferramenta para somatória e análise de arquivos XML de NF-e.".to_string(),
            active: true,
        },
        SystemInfo {
            id: "certificados-digitais".to_string(),
            slug: "certificados-digitais".to_string(),
            name: "Certificados Digitais".to_string(),
            description: "Emissão e gestão de certificados digitais (A1, A3).".to_string(),
            active: true,
        },
    ]
}

pub fn find_system_by_slug(slug: &str) -> Option<SystemInfo> {
    get_all_systems().into_iter().find(|s| s.slug == slug)
}
