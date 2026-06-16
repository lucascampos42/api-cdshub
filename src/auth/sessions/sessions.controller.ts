import { Controller, Get, Delete, Param, UseGuards, Req } from '@nestjs/common';
import { SessionsService } from './sessions.service';
import { JwtAuthGuard } from '../guards/jwt-auth.guard';
import { ApiTags, ApiOperation, ApiBearerAuth } from '@nestjs/swagger';

@ApiTags('Sessions')
@ApiBearerAuth()
@UseGuards(JwtAuthGuard)
@Controller('auth/sessions')
export class SessionsController {
  constructor(private readonly sessionsService: SessionsService) {}

  @Get()
  @ApiOperation({ summary: 'Listar todas as sessões ativas do usuário' })
  async list(@Req() req) {
    const userId = req.user.sub;
    return this.sessionsService.listSessions(userId);
  }

  @Delete(':id')
  @ApiOperation({ summary: 'Revogar uma sessão específica' })
  async revoke(@Req() req, @Param('id') sessionId: string) {
    const userId = req.user.sub;
    await this.sessionsService.revokeSession(userId, sessionId);
    return { success: true, message: 'Sessão revogada com sucesso' };
  }

  @Delete()
  @ApiOperation({
    summary: 'Revogar todas as sessões do usuário (Logout Global)',
  })
  async revokeAll(@Req() req) {
    const userId = req.user.sub;
    await this.sessionsService.revokeAllSessions(userId);
    return { success: true, message: 'Todas as sessões foram revogadas' };
  }
}
