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
import { UserContextInterceptor } from './common/context/user-context.interceptor';
import { CaslModule } from './casl/casl.module';
import { PoliciesGuard } from './casl/policies.guard';

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
  ],
  controllers: [],
  providers: [
    {
      provide: APP_GUARD,
      useClass: ScopedApiKeyGuard,
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
