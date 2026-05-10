import { IsString, IsNotEmpty, IsOptional, IsEnum } from 'class-validator';
import { ApiProperty } from '@nestjs/swagger';
import { SuggestionStatus } from '@prisma/client';

export { SuggestionStatus };

export class CreateSuggestionDto {
  @ApiProperty({ description: 'Título da sugestão' })
  @IsString()
  @IsNotEmpty()
  title!: string;

  @ApiProperty({ description: 'Descrição detalhada' })
  @IsString()
  @IsNotEmpty()
  description!: string;

  @ApiProperty({ description: 'Sistema relacionado (Ex: CDS Gestor)' })
  @IsString()
  @IsNotEmpty()
  system!: string;
}

export class UpdateSuggestionStatusDto {
  @ApiProperty({ enum: SuggestionStatus })
  @IsEnum(SuggestionStatus)
  status!: SuggestionStatus;
}
