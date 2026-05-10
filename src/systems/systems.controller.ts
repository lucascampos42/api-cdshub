import {
  Controller,
  Get,
  Post,
  Put,
  Delete,
  Body,
  Param,
  UseGuards,
  Req,
  ForbiddenException,
} from '@nestjs/common';
import { SystemsService } from './systems.service';
import { JwtAuthGuard } from '../auth/guards/jwt-auth.guard';
import { ApiTags, ApiBearerAuth, ApiOperation } from '@nestjs/swagger';

@ApiTags('Sistemas')
@ApiBearerAuth()
@UseGuards(JwtAuthGuard)
@Controller('systems')
export class SystemsController {
  constructor(private readonly systemsService: SystemsService) {}

  // --- Master Endpoints (SuperAdmin Only) ---

  @Get()
  @ApiOperation({ summary: 'Listar todos os sistemas (Master)' })
  async findAll(@Req() req) {
    this.checkSuperAdmin(req.user);
    return this.systemsService.findAll();
  }

  @Post()
  @ApiOperation({ summary: 'Criar novo sistema (Master)' })
  async create(@Req() req, @Body() data: any) {
    this.checkSuperAdmin(req.user);
    return this.systemsService.create(data);
  }

  @Post('revenda/:revendaId/:systemId')
  @ApiOperation({ summary: 'Liberar sistema para uma revenda (Master)' })
  async assignToRevenda(
    @Req() req,
    @Param('revendaId') revendaId: string,
    @Param('systemId') systemId: string,
  ) {
    this.checkSuperAdmin(req.user);
    return this.systemsService.assignToRevenda(revendaId, systemId);
  }

  @Delete('revenda/:revendaId/:systemId')
  @ApiOperation({ summary: 'Remover sistema de uma revenda (Master)' })
  async unassignFromRevenda(
    @Req() req,
    @Param('revendaId') revendaId: string,
    @Param('systemId') systemId: string,
  ) {
    this.checkSuperAdmin(req.user);
    return this.systemsService.unassignFromRevenda(revendaId, systemId);
  }

  // --- Reseller Endpoints ---

  @Get('revenda/:revendaId')
  @ApiOperation({ summary: 'Listar sistemas liberados para a revenda' })
  async findByRevenda(@Req() req, @Param('revendaId') revendaId: string) {
    // Apenas a própria revenda ou SuperAdmin
    if (req.user.userType !== 'CODESDEVS_SUPERADMIN' && req.user.revendaId !== revendaId) {
      throw new ForbiddenException('Acesso negado');
    }
    return this.systemsService.findByRevenda(revendaId);
  }

  @Post('company/:companyId/:systemId')
  @ApiOperation({ summary: 'Ativar/Desativar sistema para uma empresa (Revenda)' })
  async toggleForCompany(
    @Req() req,
    @Param('companyId') companyId: string,
    @Param('systemId') systemId: string,
    @Body('active') active: boolean,
  ) {
    // Lógica simplificada: apenas SuperAdmin ou se for da Revenda (validação no service)
    return this.systemsService.toggleForCompany(companyId, systemId, active);
  }

  @Get('company/:companyId')
  @ApiOperation({ summary: 'Listar sistemas ativos para a empresa' })
  async findByCompany(@Req() req, @Param('companyId') companyId: string) {
    return this.systemsService.findByCompany(companyId);
  }

  private checkSuperAdmin(user: any) {
    if (user.userType !== 'CODESDEVS_SUPERADMIN') {
      throw new ForbiddenException('Apenas SuperAdmin CodesDevs pode realizar esta ação');
    }
  }
}
