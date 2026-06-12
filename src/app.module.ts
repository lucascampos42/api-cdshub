import { Module } from '@nestjs/common';
import { ConfigModule, ConfigService } from '@nestjs/config';
import { APP_GUARD, APP_INTERCEPTOR } from '@nestjs/core';
import { BullModule } from '@nestjs/bullmq';
import { PrismaModule } from './prisma/prisma.module';
import { AuthModule } from './auth/auth.module';
import { UsersModule } from './users/users.module';
import { RevendasModule } from './revendas/revendas.module';
import { HealthModule } from './health/health.module';
import { ScopedApiKeyGuard } from './auth/guards/scoped-api-key.guard';
import { UserContextService } from './common/context/user-context.service';
import { UserContextInterceptor } from './common/context/user-context.interceptor';
import { CaslModule } from './casl/casl.module';
import { JwtAuthGuard } from './auth/guards/jwt-auth.guard';
import { PoliciesGuard } from './casl/policies.guard';
import { SuggestionsModule } from './suggestions/suggestions.module';
import { SystemsModule } from './systems/systems.module';
import { SessionsModule } from './auth/sessions/sessions.module';
import { CompaniesModule } from './companies/companies.module';
import { ClientsModule } from './clients/clients.module';

@Module({
  imports: [
    ConfigModule.forRoot({
      isGlobal: true,
    }),
    BullModule.forRootAsync({
      imports: [ConfigModule],
      useFactory: async (configService: ConfigService) => ({
        connection: {
          host: configService.get('REDIS_HOST', 'localhost'),
          port: configService.get('REDIS_PORT', 6379),
        },
      }),
      inject: [ConfigService],
    }),
    PrismaModule,
    AuthModule,
    UsersModule,
    RevendasModule,
    HealthModule,
    CaslModule,
    SuggestionsModule,
    SystemsModule,
    SessionsModule,
    CompaniesModule,
    ClientsModule,
  ],
  controllers: [],
  providers: [
    {
      provide: APP_GUARD,
      useClass: ScopedApiKeyGuard,
    },
    {
      provide: APP_GUARD,
      useClass: JwtAuthGuard,
    },
    {
      provide: APP_GUARD,
      useClass: PoliciesGuard,
    },
    {
      provide: APP_INTERCEPTOR,
      useClass: UserContextInterceptor,
    },
  ],
})
export class AppModule {}
