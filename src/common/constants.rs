pub mod ticket {
    pub mod status {
        pub const AGUARDANDO_ATENDIMENTO: &str = "AGUARDANDO_ATENDIMENTO";
        pub const CONCLUIDO: &str = "CONCLUIDO";
        pub const CANCELADO: &str = "CANCELADO";

        pub fn is_closed(s: &str) -> bool {
            s == CONCLUIDO || s == CANCELADO
        }
    }

    pub mod priority {
        pub const MEDIA: &str = "MEDIA";
    }
}

pub mod suggestion {
    pub mod status {
        pub const ABERTO: &str = "ABERTO";
    }
}
