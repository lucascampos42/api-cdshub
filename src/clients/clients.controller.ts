import {
  Controller,
  Get,
  Post,
  Body,
  Patch,
  Param,
  Delete,
  UseGuards,
  Query,
  Request,
} from '@nestjs/common';
import { ClientsService } from './clients.service';
import { CreateClientDto } from './dto/create-client.dto';
import { ApiTags, ApiOperation, ApiBearerAuth } from '@nestjs/swagger';
import { JwtAuthGuard } from '../auth/guards/jwt-auth.guard';
import { CheckPolicies } from '../casl/check-policies.decorator';
import { Action } from '../casl/casl-ability.factory';

@ApiTags('Clients')
@ApiBearerAuth()
@UseGuards(JwtAuthGuard)
@Controller('clients')
export class ClientsController {
  constructor(private readonly clientsService: ClientsService) {}

  @Post()
  @CheckPolicies((ability) => ability.can(Action.Create, 'Client'))
  @ApiOperation({ summary: 'Criar um novo cliente (Contratante)' })
  create(@Body() createClientDto: CreateClientDto, @Request() req) {
    // Se for REVENDA_ADMIN, injeta o revendaId dele se não vier no DTO
    if (req.user.userType === 'REVENDA_ADMIN' && !createClientDto.revendaId) {
      createClientDto.revendaId = req.user.revendaId;
    }
    return this.clientsService.create(createClientDto);
  }

  @Get()
  @CheckPolicies((ability) => ability.can(Action.Read, 'Client'))
  @ApiOperation({ summary: 'Listar todos os clientes' })
  findAll(@Query('revendaId') revendaId?: string, @Request() req?) {
    // Se for REVENDA_ADMIN, só vê os seus
    if (req.user.userType === 'REVENDA_ADMIN') {
      return this.clientsService.findAll(req.user.revendaId);
    }
    return this.clientsService.findAll(revendaId);
  }

  @Get(':id')
  @CheckPolicies((ability) => ability.can(Action.Read, 'Client'))
  @ApiOperation({ summary: 'Obter detalhes de um cliente' })
  findOne(@Param('id') id: string) {
    return this.clientsService.findOne(id);
  }

  @Patch(':id')
  @CheckPolicies((ability) => ability.can(Action.Update, 'Client'))
  @ApiOperation({ summary: 'Atualizar dados de um cliente' })
  update(@Param('id') id: string, @Body() updateData: any) {
    return this.clientsService.update(id, updateData);
  }

  @Delete(':id')
  @CheckPolicies((ability) => ability.can(Action.Delete, 'Client'))
  @ApiOperation({ summary: 'Remover um cliente' })
  remove(@Param('id') id: string) {
    return this.clientsService.remove(id);
  }
}
