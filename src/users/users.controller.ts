import {
  Controller,
  Get,
  Post,
  Body,
  Param,
  Patch,
  Delete,
  UseGuards,
  Query,
} from '@nestjs/common';
import { UsersService } from './users.service';
import { CreateUserDto } from './dto/create-user.dto';
import { ApiTags, ApiOperation, ApiBearerAuth } from '@nestjs/swagger';
import { JwtAuthGuard } from '../auth/guards/jwt-auth.guard';
import { CheckPolicies } from '../casl/check-policies.decorator';
import { Action } from '../casl/casl-ability.factory';

@ApiTags('Users')
@ApiBearerAuth()
@UseGuards(JwtAuthGuard)
@Controller('users')
export class UsersController {
  constructor(private readonly usersService: UsersService) {}

  @Post()
  @CheckPolicies((ability) => ability.can(Action.Create, 'User'))
  @ApiOperation({ summary: 'Criar um novo usuário' })
  create(@Body() createUserDto: CreateUserDto) {
    return this.usersService.create(createUserDto);
  }

  @Get()
  @CheckPolicies((ability) => ability.can(Action.Read, 'User'))
  @ApiOperation({ summary: 'Listar usuários' })
  findAll(@Query('revendaId') revendaId?: string) {
    if (revendaId) {
      return this.usersService.findAllByRevenda(revendaId);
    }
    return this.usersService.findAll();
  }

  @Get(':id')
  @CheckPolicies((ability) => ability.can(Action.Read, 'User'))
  @ApiOperation({ summary: 'Obter detalhes de um usuário' })
  findOne(@Param('id') id: string) {
    return this.usersService.findOne(id);
  }

  @Patch(':id')
  @CheckPolicies((ability) => ability.can(Action.Update, 'User'))
  @ApiOperation({ summary: 'Atualizar um usuário' })
  update(@Param('id') id: string, @Body() data: any) {
    return this.usersService.update(id, data);
  }
}
