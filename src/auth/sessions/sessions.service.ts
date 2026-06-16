import { Injectable, Logger } from '@nestjs/common';
import { ConfigService } from '@nestjs/config';
import Redis from 'ioredis';
import { v4 as uuidv4 } from 'uuid';

export interface SessionData {
  id: string;
  userId: string;
  ip: string;
  userAgent: string;
  createdAt: number;
}

@Injectable()
export class SessionsService {
  private readonly redis: Redis;
  private readonly logger = new Logger(SessionsService.name);

  constructor(private readonly configService: ConfigService) {
    this.redis = new Redis({
      host: this.configService.get('REDIS_HOST', 'localhost'),
      port: this.configService.get('REDIS_PORT', 6379),
    });
  }

  async createSession(
    userId: string,
    ip: string,
    userAgent: string,
  ): Promise<string> {
    const sessionId = uuidv4();
    const sessionData: SessionData = {
      id: sessionId,
      userId,
      ip,
      userAgent,
      createdAt: Date.now(),
    };

    // TTL de 7 dias (mesmo do Refresh Token)
    const ttl = 7 * 24 * 60 * 60;

    await this.redis.set(
      `session:${userId}:${sessionId}`,
      JSON.stringify(sessionData),
      'EX',
      ttl,
    );

    this.logger.log(`Nova sessão criada: ${sessionId} para usuário: ${userId}`);
    return sessionId;
  }

  async listSessions(userId: string): Promise<SessionData[]> {
    const keys = await this.redis.keys(`session:${userId}:*`);
    if (keys.length === 0) return [];

    const sessions = await this.redis.mget(...keys);
    return sessions.filter((s): s is string => !!s).map((s) => JSON.parse(s));
  }

  async revokeSession(userId: string, sessionId: string): Promise<void> {
    await this.redis.del(`session:${userId}:${sessionId}`);
    this.logger.log(`Sessão revogada: ${sessionId} para usuário: ${userId}`);
  }

  async revokeAllSessions(userId: string): Promise<void> {
    const keys = await this.redis.keys(`session:${userId}:*`);
    if (keys.length > 0) {
      await this.redis.del(...keys);
    }
    this.logger.log(`Todas as sessões revogadas para o usuário: ${userId}`);
  }

  async isSessionValid(userId: string, sessionId: string): Promise<boolean> {
    const session = await this.redis.get(`session:${userId}:${sessionId}`);
    return !!session;
  }
}
