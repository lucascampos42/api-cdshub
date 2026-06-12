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
} from '@nestjs/common';
import { CompaniesService } from './companies.service';
import { CreateCompanyDto } from './dto/create-company.dto';
import { ApiTags, ApiOperation, ApiBearerAuth } from '@nestjs/swagger';
import { JwtAuthGuard } from '../auth/guards/jwt-auth.guard';
import { CheckPolicies } from '../casl/check-policies.decorator';
import { Action } from '../casl/casl-ability.factory';

@ApiTags('Companies')
@ApiBearerAuth()
@UseGuards(JwtAuthGuard)
@Controller('companies')
export class CompaniesController {
  constructor(private readonly companiesService: CompaniesService) {}

  @Post()
  @CheckPolicies((ability) => ability.can(Action.Create, 'Company'))
  @ApiOperation({ summary: 'Criar uma nova empresa (Cliente da Revenda)' })
  create(@Body() createCompanyDto: CreateCompanyDto) {
    return this.companiesService.create(createCompanyDto);
  }

  @Get()
  @CheckPolicies((ability) => ability.can(Action.Read, 'Company'))
  @ApiOperation({ summary: 'Listar todas as empresas' })
  findAll(@Query('revendaId') revendaId?: string) {
    return this.companiesService.findAll(revendaId);
  }

  @Get(':id')
  @CheckPolicies((ability) => ability.can(Action.Read, 'Company'))
  @ApiOperation({ summary: 'Obter detalhes de uma empresa' })
  findOne(@Param('id') id: string) {
    return this.companiesService.findOne(id);
  }

  @Patch(':id')
  @CheckPolicies((ability) => ability.can(Action.Update, 'Company'))
  @ApiOperation({ summary: 'Atualizar dados de uma empresa' })
  update(@Param('id') id: string, @Body() updateData: any) {
    return this.companiesService.update(id, updateData);
  }

  @Delete(':id')
  @CheckPolicies((ability) => ability.can(Action.Delete, 'Company'))
  @ApiOperation({ summary: 'Inativar uma empresa' })
  remove(@Param('id') id: string) {
    return this.companiesService.remove(id);
  }
}
