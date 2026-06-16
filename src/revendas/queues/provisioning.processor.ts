import { Processor, WorkerHost } from '@nestjs/bullmq';
import { Logger } from '@nestjs/common';
import { Job } from 'bullmq';
import { ConfigService } from '@nestjs/config';

@Processor('provisioning-queue')
export class ProvisioningProcessor extends WorkerHost {
  private readonly logger = new Logger(ProvisioningProcessor.name);

  constructor(private readonly configService: ConfigService) {
    super();
  }

  async process(job: Job<{ schemaName: string }>): Promise<any> {
    const { schemaName } = job.data;
    this.logger.log(
      `Processando provisionamento assíncrono para: ${schemaName}`,
    );

    try {
      await this.callRevendaProvisioning(schemaName);
      this.logger.log(
        `Provisionamento concluído com sucesso para: ${schemaName}`,
      );
      return { success: true };
    } catch (error) {
      this.logger.error(
        `Falha ao processar job de provisionamento: ${(error as any).message}`,
      );
      throw error; // Lançar erro permite que o BullMQ tente novamente conforme configurado
    }
  }

  private async callRevendaProvisioning(schemaName: string) {
    const revendaApiUrl = this.configService.get<string>('REVENDA_API_URL');
    const internalApiKey =
      this.configService.get<string>('REVENDA_API_KEY') ||
      this.configService.get<string>('INTERNAL_API_KEY');

    if (!revendaApiUrl) {
      throw new Error('REVENDA_API_URL não configurada no api-flow');
    }

    const response = await fetch(`${revendaApiUrl}/internal/provisioning`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'x-api-key': internalApiKey || '',
      },
      body: JSON.stringify({ schemaName }),
    });

    if (!response.ok) {
      const errorData = await response
        .json()
        .catch(() => ({ message: 'Erro desconhecido na API de Revenda' }));
      throw new Error(errorData.message || `HTTP ${response.status}`);
    }

    return true;
  }
}
