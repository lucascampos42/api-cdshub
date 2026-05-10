import { Injectable, NotFoundException } from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { CreateClientDto } from './dto/create-client.dto';

@Injectable()
export class ClientsService {
  constructor(private prisma: PrismaService) {}

  async create(dto: CreateClientDto) {
    const { systemIds, ...clientData } = dto;

    return this.prisma.$transaction(async (tx) => {
      // 1. Criar o Cliente
      const client = await tx.client.create({
        data: clientData,
      });

      // 2. Se houver sistemas selecionados, criamos uma empresa padrão para o cliente
      if (systemIds && systemIds.length > 0) {
        const generatedSubdomain = client.name
          .replace(/[^a-zA-Z0-9]/g, '_')
          .toLowerCase();

        const company = await tx.company.create({
          data: {
            clientId: client.id,
            revendaId: client.revendaId,
            name: client.name,
            subdomain: generatedSubdomain,
            schemaName: `company_${generatedSubdomain}`,
            dbConnectionString: process.env.DATABASE_URL || '',
            active: true,
            // Copiar dados de contato/endereço do cliente para a empresa
            email: client.email,
            phone: client.phone,
            document: client.document,
            documentType: client.documentType,
            zipCode: client.zipCode,
            street: client.street,
            number: client.number,
            complement: client.complement,
            neighborhood: client.neighborhood,
            city: client.city,
            state: client.state,
          },
        });

        // 3. Vincular os sistemas à empresa criada
        await tx.companySystem.createMany({
          data: systemIds.map((slug) => ({
            companyId: company.id,
            systemSlug: slug,
            active: true,
          })),
        });
      }

      return client;
    });
  }

  async findAll(revendaId?: string) {
    return this.prisma.client.findMany({
      where: revendaId ? { revendaId } : {},
      include: {
        _count: {
          select: { companies: true },
        },
      },
      orderBy: { createdAt: 'desc' },
    });
  }

  async findOne(id: string) {
    const client = await this.prisma.client.findUnique({
      where: { id },
      include: {
        companies: true,
      },
    });

    if (!client) {
      throw new NotFoundException('Cliente não encontrado');
    }

    return client;
  }

  async update(id: string, dto: any) {
    return this.prisma.client.update({
      where: { id },
      data: dto,
    });
  }

  async remove(id: string) {
    return this.prisma.client.delete({
      where: { id },
    });
  }
}
