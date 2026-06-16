import {
  Injectable,
  Logger,
  ConflictException,
  InternalServerErrorException,
} from '@nestjs/common';
import { PrismaService } from '../prisma/prisma.service';
import { CreateRevendaDto } from './dto/create-revenda.dto';
import { ConfigService } from '@nestjs/config';
import { InjectQueue } from '@nestjs/bullmq';
import { Queue } from 'bullmq';

@Injectable()
export class RevendasService {
  private readonly logger = new Logger(RevendasService.name);

  constructor(
    private prisma: PrismaService,
    private configService: ConfigService,
    @InjectQueue('provisioning-queue') private provisioningQueue: Queue,
  ) {}

  async create(createRevendaDto: CreateRevendaDto) {
    const { provisionNow, ...data } = createRevendaDto;

    try {
      const revenda = await this.prisma.revenda.create({
        data,
      });

      // Gerar o schemaName da Revenda (ex: revenda_dominio)
      const schemaName = `revenda_${revenda.domain.replace(/[^a-zA-Z0-9]/g, '_').toLowerCase()}`;

      if (provisionNow !== false) {
        try {
          this.logger.log(
            `Adicionando job de provisionamento para ${schemaName} na fila.`,
          );
          await this.provisioningQueue.add('provision', { schemaName });
        } catch (error) {
          this.logger.error(
            `Erro ao enfileirar provisionamento: ${(error as any).message}`,
          );
          // Não deletamos a revenda aqui, pois o job pode ser retentado ou disparado manualmente depois
        }
      }

      return revenda;
    } catch (error) {
      if (
        error instanceof ConflictException ||
        error instanceof InternalServerErrorException
      ) {
        throw error;
      }
      if ((error as any).code === 'P2002') {
        throw new ConflictException(
          'Revenda com este domínio ou documento já existe.',
        );
      }
      throw new InternalServerErrorException('Erro ao criar revenda');
    }
  }

  async findAll() {
    return this.prisma.revenda.findMany();
  }

  async findOne(id: string) {
    return this.prisma.revenda.findUnique({ where: { id } });
  }
}
