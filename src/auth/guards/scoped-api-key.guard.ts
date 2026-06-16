import {
  Injectable,
  CanActivate,
  ExecutionContext,
  UnauthorizedException,
} from '@nestjs/common';
import { Reflector } from '@nestjs/core';
import { ConfigService } from '@nestjs/config';
import {
  SERVICE_SCOPE_KEY,
  ServiceScopeType,
} from '../decorators/service-scope.decorator';

@Injectable()
export class ScopedApiKeyGuard implements CanActivate {
  constructor(
    private reflector: Reflector,
    private configService: ConfigService,
  ) {}

  canActivate(context: ExecutionContext): boolean {
    const scope = this.reflector.getAllAndOverride<ServiceScopeType>(
      SERVICE_SCOPE_KEY,
      [context.getHandler(), context.getClass()],
    );

    if (!scope) {
      return true; // Se não houver escopo definido, não bloqueia por API Key (outros guards cuidarão)
    }

    const request = context.switchToHttp().getRequest();
    const apiKey = request.headers['x-api-key'];

    if (!apiKey) {
      throw new UnauthorizedException('API Key ausente');
    }

    // Mapeamento de escopos para chaves de ambiente
    const keyMap: Record<ServiceScopeType, string> = {
      REVENDA: 'REVENDA_API_KEY',
      CDSGESTOR: 'CDSGESTOR_API_KEY',
      FLOW: 'FLOW_API_KEY',
      ANY_INTERNAL: 'INTERNAL_API_KEY',
    };

    const envKeyName = keyMap[scope];
    const expectedKey =
      this.configService.get<string>(envKeyName) ||
      this.configService.get<string>('INTERNAL_API_KEY');

    if (apiKey !== expectedKey) {
      throw new UnauthorizedException('API Key inválida para este escopo');
    }

    return true;
  }
}
