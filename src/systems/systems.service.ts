import { Injectable, NotFoundException } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { SYSTEMS, System } from '../common/constants/systems.constants';

@Injectable()
export class SystemsService {
  constructor(private prisma: PrismaService) {}

  // --- Master (Hardcoded Systems) ---

  async findAll(): Promise<System[]> {
    return SYSTEMS;
  }

  async findOne(slug: string): Promise<System> {
    const system = SYSTEMS.find((s) => s.slug === slug);
    if (!system) throw new NotFoundException('Sistema não encontrado');
    return system;
  }

  // creation and update removed as they are hardcoded

  // --- Revenda Systems Provisioning ---

  async findByRevenda(revendaId: string) {
    const revendaSystems = await this.prisma.revendaSystem.findMany({
      where: { revendaId },
    });

    // Merge with hardcoded data
    return SYSTEMS.filter((s) =>
      revendaSystems.some((rs) => rs.systemSlug === s.slug),
    );
  }

  async assignToRevenda(revendaId: string, systemSlug: string) {
    // Validate if slug exists
    await this.findOne(systemSlug);

    return this.prisma.revendaSystem.upsert({
      where: { revendaId_systemSlug: { revendaId, systemSlug } },
      create: { revendaId, systemSlug },
      update: {},
    });
  }

  async unassignFromRevenda(revendaId: string, systemSlug: string) {
    return this.prisma.revendaSystem.delete({
      where: { revendaId_systemSlug: { revendaId, systemSlug } },
    });
  }

  // --- Company Systems Provisioning ---

  async findByCompany(companyId: string) {
    const companySystems = await this.prisma.companySystem.findMany({
      where: { companyId },
    });

    return companySystems.map((cs) => {
      const system = SYSTEMS.find((s) => s.slug === cs.systemSlug);
      return {
        ...cs,
        system: system || { name: cs.systemSlug, slug: cs.systemSlug },
      };
    });
  }

  async toggleForCompany(companyId: string, systemSlug: string, active: boolean) {
    // Verificar se a revenda desta empresa possui o sistema
    const company = await this.prisma.company.findUnique({
      where: { id: companyId },
      select: { revendaId: true },
    });

    if (!company?.revendaId) throw new NotFoundException('Empresa ou Revenda não encontrada');

    const hasAccess = await this.prisma.revendaSystem.findUnique({
      where: { revendaId_systemSlug: { revendaId: company.revendaId, systemSlug } },
    });

    if (!hasAccess) {
      throw new ForbiddenException('A revenda não possui este sistema liberado para comercialização');
    }

    return this.prisma.companySystem.upsert({
      where: { companyId_systemSlug: { companyId, systemSlug } },
      create: { companyId, systemSlug, active },
      update: { active },
    });
  }
}

import { ForbiddenException } from '@nestjs/common';
