import { Injectable, OnModuleInit, OnModuleDestroy, Logger } from '@nestjs/common';
import { PrismaClient } from '@prisma/client';
import { Pool } from 'pg';
import { PrismaPg } from '@prisma/adapter-pg';
import { UserContextService } from '../common/context/user-context.service';

@Injectable()
export class PrismaService
  extends PrismaClient
  implements OnModuleInit, OnModuleDestroy {
  private readonly logger = new Logger(PrismaService.name);
  private extendedClient: any;

  constructor(private readonly userContext: UserContextService) {
    const connectionString = process.env.DATABASE_URL;
    const pool = new Pool({ connectionString });
    const adapter = new PrismaPg(pool);

    super({
      adapter,
      log: process.env.NODE_ENV === 'development' ? ['query', 'info', 'warn', 'error'] : ['error'],
    });

    const self = this;
    this.extendedClient = this.$extends({
      query: {
        $allModels: {
          async create({ model, args, query }) {
            const result = await query(args);
            await self.logAction('CREATE', model, result);
            return result;
          },
          async update({ model, args, query }) {
            const result = await query(args);
            await self.logAction('UPDATE', model, result);
            return result;
          },
          async delete({ model, args, query }) {
            const result = await query(args);
            await self.logAction('DELETE', model, result);
            return result;
          },
        },
      },
    });

    return this.extendedClient;
  }

  private async logAction(action: string, model: string, result: any) {
    const userId = this.userContext.getUserId();
    const sensitiveModels = ['User', 'Revenda', 'Company', 'AccessRule'];

    if (sensitiveModels.includes(model)) {
      try {
        // Usar o cliente base (this) para evitar recursividade infinita na auditoria
        // Nota: Certifique-se que o modelo 'AuditLog' existe no seu schema.prisma
        // Se o nome for diferente (ex: 'Log'), ajuste abaixo.
        if ((this as any).auditLog) {
          await (this as any).auditLog.create({
            data: {
              userId,
              action,
              target: `${model}:${result.id || 'unknown'}`,
              details: JSON.stringify(result),
              createdAt: new Date(),
            },
          });
        }
      } catch (error) {
        this.logger.error(`Erro ao gravar log de auditoria: ${(error as any).message}`);
      }
    }
  }

  async onModuleInit() {
    await this.$connect();
  }

  async onModuleDestroy() {
    await this.$disconnect();
  }
}
