import { SetMetadata } from '@nestjs/common';

export type ServiceScopeType =
  | 'REVENDA'
  | 'CDSGESTOR'
  | 'FLOW'
  | 'ANY_INTERNAL';

export const SERVICE_SCOPE_KEY = 'service_scope';
export const ServiceScope = (scope: ServiceScopeType) =>
  SetMetadata(SERVICE_SCOPE_KEY, scope);
