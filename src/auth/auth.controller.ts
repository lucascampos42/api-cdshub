import {
  Body,
  Controller,
  Post,
  Res,
  HttpStatus,
  UseGuards,
  Req,
  Get,
  UnauthorizedException,
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

    const result = await this.authService.login(loginDto, ip, userAgent);

    if ('requires2FA' in result) {
      return result;
    }

    const { access_token, refresh_token, user, companies, currentCompany } = result as any;

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

// --- 2FA ENDPOINTS ---

@UseGuards(JwtAuthGuard)
@ApiBearerAuth()
@Post('2fa/generate')
@ApiOperation({ summary: 'Gerar segredo e QR Code para 2FA' })
async generate2FA(@Req() req) {
  const userId = req.user['userId'];
  return this.authService.generate2FASecret(userId);
}

@UseGuards(JwtAuthGuard)
@ApiBearerAuth()
@Post('2fa/turn-on')
@ApiOperation({ summary: 'Ativar 2FA para o usuário' })
async turnOn2FA(@Req() req, @Body('code') code: string) {
  const userId = req.user['userId'];
  return this.authService.turnOn2FA(userId, code);
}

@UseGuards(JwtAuthGuard)
@ApiBearerAuth()
@Post('2fa/turn-off')
@ApiOperation({ summary: 'Desativar 2FA para o usuário' })
async turnOff2FA(@Req() req) {
  const userId = req.user['userId'];
  return this.authService.turnOff2FA(userId);
}

@Public()
@Post('login/verify-2fa')
@ApiOperation({ summary: 'Verificar código 2FA após login inicial' })
async verify2FA(
  @Req() req: Request,
  @Body('tempToken') tempToken: string,
  @Body('code') code: string,
) {
  // Decodificar o tempToken para pegar o userId
  let userId: string;
  try {
    const payload = await this.authService['jwtService'].verifyAsync(tempToken);
    if (payload.type !== '2fa_pending') throw new Error();
    userId = payload.sub;
  } catch (e) {
    throw new UnauthorizedException('Token temporário inválido ou expirado');
  }

  const ip = (req.headers['x-forwarded-for'] as string) || (req as any).socket.remoteAddress || 'unknown';
  const userAgent = req.headers['user-agent'] || 'unknown';

  return this.authService.verify2FA(userId, code, ip, userAgent);
}

/**
 * ✨ Trocar empresa atual
...
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
