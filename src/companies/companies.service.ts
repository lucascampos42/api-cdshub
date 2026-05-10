import {
  Injectable,
  Logger,
  ConflictException,
  InternalServerErrorException,
} from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { CreateCompanyDto } from './dto/create-company.dto';
import { Prisma } from '@prisma/client';
import { ConfigService } from '@nestjs/config';

@Injectable()
export class CompaniesService {
  private readonly logger = new Logger(CompaniesService.name);

  constructor(
    private prisma: PrismaService,
    private configService: ConfigService,
  ) {}

  async create(createCompanyDto: CreateCompanyDto) {
    try {
      const generatedSubdomain = (
        createCompanyDto.subdomain || createCompanyDto.name
      )
        .replace(/[^a-zA-Z0-9]/g, '_')
        .toLowerCase();

      const data: Prisma.CompanyUncheckedCreateInput = {
        revendaId: createCompanyDto.revendaId,
        clientId: createCompanyDto.clientId,
        name: createCompanyDto.name,
        subdomain: generatedSubdomain,
        schemaName: `company_${generatedSubdomain}`,
        dbConnectionString: process.env.DATABASE_URL || '',
        customDomain: createCompanyDto.customDomain,

        // Contato
        email: createCompanyDto.email,
        phone: createCompanyDto.phone,

        // Documentos
        ...(createCompanyDto.document && {
          document: createCompanyDto.document,
        }),
        ...(createCompanyDto.documentType && {
          documentType: createCompanyDto.documentType,
        }),

        // Endereço
        zipCode: createCompanyDto.zipCode,
        street: createCompanyDto.street,
        number: createCompanyDto.number,
        complement: createCompanyDto.complement,
        neighborhood: createCompanyDto.neighborhood,
        city: createCompanyDto.city,
        state: createCompanyDto.state,

        // Financeiro
        monthlyFee: createCompanyDto.monthlyFee,
        billingDay: createCompanyDto.billingDay,
        paymentMethod: createCompanyDto.paymentMethod,

        // Fiscal
        taxRegime: createCompanyDto.taxRegime,
        accountantName: createCompanyDto.accountantName,
        accountantEmail: createCompanyDto.accountantEmail,
        stateRegistration: createCompanyDto.stateRegistration,
        municipalRegistration: createCompanyDto.municipalRegistration,

        // Detalhes do Negócio
        tradeName: createCompanyDto.tradeName,
        segment: createCompanyDto.segment,
        openingDate: createCompanyDto.openingDate ? new Date(createCompanyDto.openingDate) : undefined,
        openingHours: createCompanyDto.openingHours,
        modules: createCompanyDto.modules,

        // Relacionamentos
        parentCompanyId: createCompanyDto.parentCompanyId,
        ownerUserId: createCompanyDto.ownerUserId,

        // Configuração
        active: createCompanyDto.active ?? true,
      };

      if (createCompanyDto.startInDemoMode || createCompanyDto.isDemoMode) {
        const now = new Date();
        const demoEnd = new Date();
        demoEnd.setDate(demoEnd.getDate() + 30);

        data.isDemoMode = true;
        data.demoStartDate = now;
        data.demoEndDate = demoEnd;
      }

      // ✨ Validação manual de unicidade para revendaId = null
      if (!data.revendaId) {
        const existing = await this.prisma.company.findFirst({
          where: {
            subdomain: data.subdomain,
            revendaId: null,
          },
        });
        if (existing) {
          throw new ConflictException(
            'Já existe uma empresa com este subdomínio (Cliente Direto).',
          );
        }
      }

      const company = await this.prisma.company.create({
        data,
        include: {
          revenda: {
            select: {
              id: true,
              name: true,
              domain: true,
            },
          },
        },
      });

      // 🚀 Provisionamento Opcional via Chamada Interna para CDS GESTOR
      if (createCompanyDto.provisionNow !== false && company.schemaName) {
        try {
          await this.callCdsGestorProvisioning(company.schemaName);
        } catch (error: any) {
          this.logger.error(`Falha ao provisionar schema: ${error.message}`);
          // Nota: Em produção você pode querer disparar um evento para retry em vez de falhar a criação
        }
      }

      return company;
    } catch (error) {
      if (error instanceof ConflictException || error instanceof InternalServerErrorException) {
        throw error;
      }
      if (error.code === 'P2002') {
        throw new ConflictException(
          'Já existe uma empresa com este nome, subdomínio ou documento (CNPJ/CPF).',
        );
      }
      this.logger.error(`Erro ao criar empresa: ${error.message}`, error.stack);
      throw new InternalServerErrorException(
        'Erro interno ao criar empresa. Verifique os logs.',
      );
    }
  }

  private async callCdsGestorProvisioning(schemaName: string) {
    const cdsGestorUrl = this.configService.get<string>('CDSGESTOR_API_URL');
    const internalApiKey = this.configService.get<string>('INTERNAL_API_KEY');

    if (!cdsGestorUrl) {
      this.logger.warn('CDSGESTOR_API_URL não configurada no ambiente. Pulando provisionamento.');
      return;
    }

    this.logger.log(`Solicitando provisionamento para ${schemaName} em ${cdsGestorUrl}`);

    try {
      const response = await fetch(`${cdsGestorUrl}/internal/provisioning`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'x-api-key': internalApiKey || '',
        },
        body: JSON.stringify({ schemaName }),
      });

      if (!response.ok) {
        throw new Error(`Erro HTTP ${response.status}`);
      }
      this.logger.log(`Provisionamento de ${schemaName} concluído com sucesso via API.`);
    } catch (err) {
      this.logger.error(`Erro ao chamar API de provisionamento: ${err.message}`);
      throw err;
    }
  }

  async findAll(revendaId?: string) {
    return this.prisma.company.findMany({
      where: revendaId ? { revendaId } : {},
      include: {
        revenda: {
          select: {
            id: true,
            name: true,
            domain: true,
          },
        },
      },
    });
  }

  async findOne(id: string) {
    return this.prisma.company.findUnique({
      where: { id },
      include: {
        revenda: true,
        userCompanies: {
          include: {
            user: {
              select: {
                id: true,
                name: true,
                email: true,
                userType: true,
              }
            }
          }
        }
      },
    });
  }

  async update(id: string, data: Prisma.CompanyUpdateInput) {
    return this.prisma.company.update({
      where: { id },
      data,
    });
  }

  async remove(id: string) {
    return this.prisma.company.update({
      where: { id },
      data: { active: false },
    });
  }
}
