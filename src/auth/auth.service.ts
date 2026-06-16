import {
  Injectable,
  UnauthorizedException,
  ForbiddenException,
} from '@nestjs/common';
import { UsersService } from '../users/users.service';
import { JwtService } from '@nestjs/jwt';
import * as bcrypt from 'bcrypt';
import { LoginDto } from './dto/login.dto';
import { ChangePasswordDto } from './dto/change-password.dto';
import { ConfigService } from '@nestjs/config';
import { PrismaService } from '../prisma/prisma.service';
import { SessionsService } from './sessions/sessions.service';

@Injectable()
export class AuthService {
  constructor(
    private usersService: UsersService,
    private jwtService: JwtService,
    private configService: ConfigService,
    private prisma: PrismaService,
    private sessionsService: SessionsService,
  ) {}

  async login(loginDto: LoginDto, ip: string, userAgent: string) {
    const user = await this.usersService.findByIdentifier(loginDto.identifier);

    if (!user || !user.passwordHash) {
      throw new UnauthorizedException('Credenciais inválidas');
    }

    const isPasswordValid = await bcrypt.compare(
      loginDto.password,
      user.passwordHash,
    );

    if (!isPasswordValid) {
      throw new UnauthorizedException('Credenciais inválidas');
    }

    let companies: any[] = [];
    let currentCompany: any = null;
    let currentCompanyId: string | null = null;

    const isClientUser = user.userType?.startsWith('CLIENTE_');

    if (isClientUser) {
      const userCompanies = await this.prisma.userCompany.findMany({
        where: { userId: user.id },
        include: { company: true },
      });

      const defaultCompany =
        userCompanies.find((uc) => uc.isDefault) || userCompanies[0];

      currentCompanyId = user.currentCompanyId;
      if (!currentCompanyId && defaultCompany) {
        currentCompanyId = defaultCompany.companyId;
        await this.usersService.update(user.id, { currentCompanyId });
      }

      companies = userCompanies.map((uc) => ({
        id: uc.company.id,
        name: uc.company.name,
        subdomain: uc.company.subdomain,
        document: uc.company.document,
        role: uc.role,
        isDefault: uc.isDefault,
      }));

      currentCompany = defaultCompany
        ? {
            id: defaultCompany.company.id,
            name: defaultCompany.company.name,
            subdomain: defaultCompany.company.subdomain,
          }
        : null;
    }

    // Criar sessão no Redis
    const sessionId = await this.sessionsService.createSession(user.id, ip, userAgent);

    const tokens = await this.getTokens(
      user.id,
      user.email,
      user,
      currentCompanyId,
      sessionId,
    );
    await this.updateRefreshToken(user.id, tokens.refreshToken);

    return {
      access_token: tokens.accessToken,
      refresh_token: tokens.refreshToken,
      user: {
        id: user.id,
        name: user.name,
        email: user.email,
        userType: user.userType,
        role: user.role,
        mustChangePassword: user.mustChangePassword,
      },
      companies,
      currentCompany,
    };
  }

  async logout(userId: string, sessionId?: string) {
    if (sessionId) {
      await this.sessionsService.revokeSession(userId, sessionId);
    } else {
      await this.sessionsService.revokeAllSessions(userId);
    }
    return this.usersService.update(userId, { hashedRefreshToken: null });
  }

  async getCompaniesContextFromJwt(userJwt: {
    userId: string;
    userType: string;
    revendaId?: string;
    companyId?: string | null;
  }) {
    const { userId, userType, revendaId, companyId } = userJwt;

    if (userType?.startsWith('CLIENTE_')) {
      const userCompanies = await this.prisma.userCompany.findMany({
        where: { userId },
        include: { company: true },
      });

      const companies = userCompanies.map((uc) => ({
        id: uc.company.id,
        name: uc.company.name,
        subdomain: uc.company.subdomain,
        document: uc.company.document,
        parentCompanyId: uc.company.parentCompanyId,
        revendaId: uc.company.revendaId,
        active: uc.company.active,
        role: uc.role,
        isDefault: uc.isDefault,
      }));

      const current =
        companies.find((c) => c.id === companyId) ||
        companies.find((c) => c.isDefault) ||
        companies[0] ||
        null;

      return {
        companies,
        currentCompany: current
          ? { id: current.id, name: current.name, subdomain: current.subdomain }
          : null,
      };
    }

    if (
      userType === 'CODESDEVS_SUPERADMIN' ||
      userType === 'CODESDEVS_SUPORTE'
    ) {
      const companies = await this.prisma.company.findMany({
        where: { active: true },
      });

      return {
        companies: companies.map((c) => ({
          id: c.id,
          name: c.name,
          subdomain: c.subdomain,
          document: c.document,
          parentCompanyId: c.parentCompanyId,
          revendaId: c.revendaId,
          active: c.active,
          role: null,
          isDefault: false,
        })),
        currentCompany: null,
      };
    }

    if (userType?.startsWith('REVENDA_') && revendaId) {
      const companies = await this.prisma.company.findMany({
        where: { revendaId, active: true },
      });

      return {
        companies: companies.map((c) => ({
          id: c.id,
          name: c.name,
          subdomain: c.subdomain,
          document: c.document,
          parentCompanyId: c.parentCompanyId,
          revendaId: c.revendaId,
          active: c.active,
          role: null,
          isDefault: false,
        })),
        currentCompany: null,
      };
    }

    return {
      companies: [],
      currentCompany: null,
    };
  }

  async switchCompany(userId: string, companyId: string, sessionId?: string) {
    const user = await this.usersService.findOne(userId);
    if (!user) {
      throw new UnauthorizedException('Usuário não encontrado');
    }

    let company;

    if (
      user.userType === 'CODESDEVS_SUPERADMIN' ||
      user.userType === 'CODESDEVS_SUPORTE'
    ) {
      company = await this.prisma.company.findUnique({
        where: { id: companyId },
      });
      if (!company || !company.active) {
        throw new ForbiddenException('Empresa não encontrada ou inativa');
      }
    } else if (user.userType?.startsWith('REVENDA_')) {
      if (!user.revendaId) {
        throw new ForbiddenException('Revenda não identificada');
      }
      company = await this.prisma.company.findFirst({
        where: { id: companyId, revendaId: user.revendaId, active: true },
      });
      if (!company) {
        throw new ForbiddenException('Acesso negado a esta empresa');
      }
    } else {
      const userCompany = await this.prisma.userCompany.findUnique({
        where: { userId_companyId: { userId, companyId } },
        include: { company: true },
      });

      if (!userCompany) {
        throw new ForbiddenException('Acesso negado a esta empresa');
      }
      company = userCompany.company;
    }

    await this.usersService.update(userId, { currentCompanyId: companyId });

    const tokens = await this.getTokens(user.id, user.email, user, companyId, sessionId);
    await this.updateRefreshToken(user.id, tokens.refreshToken);

    return {
      access_token: tokens.accessToken,
      refresh_token: tokens.refreshToken,
      currentCompany: {
        id: company.id,
        name: company.name,
        subdomain: company.subdomain,
      },
    };
  }

  async refreshTokens(userId: string, refreshToken: string, sessionId?: string) {
    const user = await this.usersService.findOne(userId);
    if (!user || !user.hashedRefreshToken)
      throw new ForbiddenException('Acesso negado');

    const refreshTokenMatches = await bcrypt.compare(
      refreshToken,
      user.hashedRefreshToken,
    );

    if (!refreshTokenMatches) throw new ForbiddenException('Acesso negado');

    // Se tiver sessionId, validar no Redis
    if (sessionId) {
      const isValid = await this.sessionsService.isSessionValid(userId, sessionId);
      if (!isValid) throw new ForbiddenException('Sessão expirada ou revogada');
    }

    const tokens = await this.getTokens(user.id, user.email, user, user.currentCompanyId, sessionId);
    await this.updateRefreshToken(user.id, tokens.refreshToken);

    return tokens;
  }

  async updateRefreshToken(userId: string, refreshToken: string) {
    const hash = await bcrypt.hash(refreshToken, 10);
    await this.usersService.update(userId, {
      hashedRefreshToken: hash,
    });
  }

  async getTokens(
    userId: string,
    email: string,
    user: any,
    companyId?: string | null,
    sessionId?: string,
  ) {
    let schemaName: string | null = null;
    let companyRole: string | null = null;

    if (companyId) {
      const company = await this.prisma.company.findUnique({
        where: { id: companyId },
        select: { subdomain: true },
      });

      const userCompany = await this.prisma.userCompany.findUnique({
        where: { userId_companyId: { userId, companyId } },
        select: { role: true },
      });

      schemaName = company?.subdomain || null;
      companyRole = userCompany?.role || null;
    }

    const payload = {
      sub: userId,
      email: email,
      userType: user.userType,
      role: user.role,
      revendaId: user.revendaId,
      companyId: companyId || null,
      schemaName: schemaName,
      companyRole: companyRole,
      sessionId: sessionId, // Incluir sessionId no token
    };

    const jwtExpiration =
      this.configService.get<string>('JWT_EXPIRATION') || '3d';
    const jwtRefreshExpiration =
      this.configService.get<string>('REFRESH_TOKEN_EXPIRATION') || '7d';

    const accessToken = await this.jwtService.signAsync(payload, {
      expiresIn: jwtExpiration,
    } as any);

    const refreshTokenSecret = this.configService.get<string>(
      'REFRESH_TOKEN_SECRET',
    );
    if (!refreshTokenSecret) {
      throw new Error('REFRESH_TOKEN_SECRET environment variable is not defined');
    }

    const refreshToken = await this.jwtService.signAsync(payload, {
      secret: refreshTokenSecret,
      expiresIn: jwtRefreshExpiration,
    } as any);

    return {
      accessToken,
      refreshToken,
    };
  }

  async changePassword(userId: string, changePasswordDto: ChangePasswordDto) {
    const user = await this.usersService.findOne(userId);
    if (!user || !user.passwordHash) {
      throw new UnauthorizedException('Usuário não encontrado');
    }

    const isPasswordValid = await bcrypt.compare(
      changePasswordDto.oldPassword,
      user.passwordHash,
    );
    if (!isPasswordValid) {
      throw new UnauthorizedException('Senha antiga incorreta');
    }

    const newPasswordHash = await bcrypt.hash(
      changePasswordDto.newPassword,
      10,
    );

    await this.usersService.update(userId, {
      passwordHash: newPasswordHash,
      mustChangePassword: false,
    });

    return { message: 'Senha alterada com sucesso' };
  }

  async dismissPasswordReminder(userId: string) {
    await this.usersService.update(userId, {
      mustChangePassword: false,
    });
    return { message: 'Lembrete desativado' };
  }
}
