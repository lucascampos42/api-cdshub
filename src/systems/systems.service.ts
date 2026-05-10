import { Injectable, NotFoundException } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';

@Injectable()
export class SystemsService {
  constructor(private prisma: PrismaService) {}

  // --- Master (Systems CRUD) ---

  async findAll() {
    return this.prisma.system.findMany({
      orderBy: { name: 'asc' },
    });
  }

  async findOne(id: string) {
    const system = await this.prisma.system.findUnique({ where: { id } });
    if (!system) throw new NotFoundException('Sistema não encontrado');
    return system;
  }

  async create(data: any) {
    return this.prisma.system.create({ data });
  }

  async update(id: string, data: any) {
    return this.prisma.system.update({ where: { id }, data });
  }

  // --- Revenda Systems Provisioning ---

  async findByRevenda(revendaId: string) {
    return this.prisma.system.findMany({
      where: {
        revendaSystems: {
          some: { revendaId },
        },
      },
    });
  }

  async assignToRevenda(revendaId: string, systemId: string) {
    return this.prisma.revendaSystem.upsert({
      where: { revendaId_systemId: { revendaId, systemId } },
      create: { revendaId, systemId },
      update: {},
    });
  }

  async unassignFromRevenda(revendaId: string, systemId: string) {
    return this.prisma.revendaSystem.delete({
      where: { revendaId_systemId: { revendaId, systemId } },
    });
  }

  // --- Company Systems Provisioning ---

  async findByCompany(companyId: string) {
    return this.prisma.companySystem.findMany({
      where: { companyId },
      include: { system: true },
    });
  }

  async toggleForCompany(companyId: string, systemId: string, active: boolean) {
    // Verificar se a revenda desta empresa possui o sistema
    const company = await this.prisma.company.findUnique({
      where: { id: companyId },
      select: { revendaId: true },
    });

    if (!company?.revendaId) throw new NotFoundException('Empresa ou Revenda não encontrada');

    const hasAccess = await this.prisma.revendaSystem.findUnique({
      where: { revendaId_systemId: { revendaId: company.revendaId, systemId } },
    });

    if (!hasAccess) {
      throw new ForbiddenException('A revenda não possui este sistema liberado para comercialização');
    }

    return this.prisma.companySystem.upsert({
      where: { companyId_systemId: { companyId, systemId } },
      create: { companyId, systemId, active },
      update: { active },
    });
  }
}

import { ForbiddenException } from '@nestjs/common';
