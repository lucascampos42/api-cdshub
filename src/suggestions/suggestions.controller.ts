import {
  Controller,
  Get,
  Post,
  Body,
  Patch,
  Param,
  Query,
  UseGuards,
  Req,
} from '@nestjs/common';
import { SuggestionsService } from './suggestions.service';
import { CreateSuggestionDto, UpdateSuggestionStatusDto } from './dto/suggestion.dto';
import { JwtAuthGuard } from '../auth/guards/jwt-auth.guard';
import { Public } from '../auth/decorators/public.decorator';
import { ApiTags, ApiOperation, ApiBearerAuth } from '@nestjs/swagger';

@ApiTags('Suggestions')
@Controller('suggestions')
export class SuggestionsController {
  constructor(private readonly suggestionsService: SuggestionsService) {}

  @Public() // Permite que visitantes vejam as sugestões
  @Get()
  @ApiOperation({ summary: 'Listar todas as sugestões' })
  findAll(@Query('system') system?: string) {
    return this.suggestionsService.findAll(system);
  }

  @UseGuards(JwtAuthGuard)
  @ApiBearerAuth()
  @Post()
  @ApiOperation({ summary: 'Criar uma nova sugestão' })
  create(@Body() createSuggestionDto: CreateSuggestionDto, @Req() req) {
    const userId = req.user?.userId;
    return this.suggestionsService.create(createSuggestionDto, userId);
  }

  @Public() // Permite voto anônimo (ou podemos restringir depois)
  @Patch(':id/vote')
  @ApiOperation({ summary: 'Votar em uma sugestão' })
  vote(@Param('id') id: string) {
    return this.suggestionsService.vote(id);
  }

  @UseGuards(JwtAuthGuard)
  @ApiBearerAuth()
  @Patch(':id/status')
  @ApiOperation({ summary: 'Atualizar o status de uma sugestão (Admin/Suporte)' })
  updateStatus(
    @Param('id') id: string,
    @Body() updateStatusDto: UpdateSuggestionStatusDto,
  ) {
    return this.suggestionsService.updateStatus(id, updateStatusDto.status);
  }
}
