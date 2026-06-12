export interface System {
  id: string; // id is kept as slug for backward compatibility if needed, or unique identifier
  slug: string;
  name: string;
  description: string;
  active: boolean;
}

export const SYSTEMS: System[] = [
  {
    id: 'cds-gestor',
    slug: 'cds-gestor',
    name: 'CDS Gestor',
    description: 'Sistema completo de gestão empresarial (ERP).',
    active: true,
  },
  {
    id: 'agenda',
    slug: 'agenda',
    name: 'Agenda',
    description: 'Sistema de agendamento e controle de visitas técnicas.',
    active: true,
  },
  {
    id: 'calculadora-xml',
    slug: 'calculadora-xml',
    name: 'Calculadora XML',
    description: 'Ferramenta para somatória e análise de arquivos XML de NF-e.',
    active: true,
  },
  {
    id: 'certificados-digitais',
    slug: 'certificados-digitais',
    name: 'Certificados Digitais',
    description: 'Emissão e gestão de certificados digitais (A1, A3).',
    active: true,
  },
];
