import {
  Body,
  Controller,
  Post,
  Res,
  HttpStatus,
  UseGuards,
  Req,
  Get,
} from '@nestjs/common';
import { AuthService } from './auth.service';
import { LoginDto } from './dto/login.dto';
import type { Response, Request } from 'express';
import {
  ApiTags,
  ApiOperation,
  ApiResponse,
  ApiBearerAuth,
} from '@nestjs/swagger';
import { Public } from './decorators/public.decorator';
import { RefreshJwtAuthGuard } from './guards/refresh-token.guard';
import { JwtAuthGuard } from './guards/jwt-auth.guard';

@ApiTags('Auth')
@Controller('auth')
export class AuthController {
  constructor(private authService: AuthService) {}

  @Public()
  @Post('login')
  @ApiOperation({ summary: 'Realizar login e obter tokens' })
  @ApiResponse({ status: 200, description: 'Login realizado com sucesso.' })
  @ApiResponse({ status: 401, description: 'Credenciais inválidas.' })
  async login(
    @Req() req: Request,
    @Body() loginDto: LoginDto,
  ) {
    const ip = (req.headers['x-forwarded-for'] as string) || req.socket.remoteAddress || 'unknown';
    const userAgent = req.headers['user-agent'] || 'unknown';

    const { access_token, refresh_token, user, companies, currentCompany } =
      await this.authService.login(loginDto, ip, userAgent);

    return {
      statusCode: HttpStatus.OK,
      message: 'Login realizado com sucesso',
      access_token,
      refresh_token,
      user,
      companies,
      currentCompany,
    };
  }

  @UseGuards(RefreshJwtAuthGuard)
  @Post('refresh')
  @ApiOperation({ summary: 'Renovar Access Token usando Refresh Token' })
  async refresh(@Req() req) {
    const userId = req.user['sub'];
    const refreshToken = req.user['refreshToken'];
    const sessionId = req.user['sessionId'];

    const tokens = await this.authService.refreshTokens(userId, refreshToken, sessionId);

    return {
      access_token: tokens.accessToken,
      refresh_token: tokens.refreshToken,
    };
  }

  @UseGuards(JwtAuthGuard)
  @ApiBearerAuth()
  @Post('logout')
  @ApiOperation({ summary: 'Encerrar sessão' })
  async logout(@Req() req) {
    const userId = req.user['userId'];
    const sessionId = req.user['sessionId'];
    
    await this.authService.logout(userId, sessionId);

    return { message: 'Logout realizado com sucesso' };
  }

  /**
   * ✨ Trocar empresa atual
   */
  @UseGuards(JwtAuthGuard)
  @ApiBearerAuth()
  @Post('switch-company')
  @ApiOperation({ summary: 'Trocar empresa atual do usuário' })
  async switchCompany(
    @Req() req,
    @Body() { companyId }: { companyId: string },
  ) {
    const userId = req.user['userId'];
    const sessionId = req.user['sessionId'];

    const { access_token, refresh_token, currentCompany } =
      await this.authService.switchCompany(userId, companyId, sessionId);

    return {
      message: 'Empresa alterada com sucesso',
      access_token,
      refresh_token,
      currentCompany,
    };
  }

  @UseGuards(JwtAuthGuard)
  @ApiBearerAuth()
  @Get('companies-context')
  @ApiOperation({
    summary: 'Obter contexto de empresas para o usuário autenticado',
  })
  async getCompaniesContext(@Req() req) {
    const { userId, userType, revendaId, companyId } = req.user;
    return this.authService.getCompaniesContextFromJwt({
      userId,
      userType,
      revendaId,
      companyId,
    });
  }
}
