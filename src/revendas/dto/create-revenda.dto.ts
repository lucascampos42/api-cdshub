import { IsString, IsNotEmpty, IsOptional, IsBoolean } from 'class-validator';
import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';

export class CreateRevendaDto {
  @ApiProperty()
  @IsString()
  @IsNotEmpty()
  name!: string;

  @ApiProperty()
  @IsString()
  @IsNotEmpty()
  domain!: string;

  @ApiProperty()
  @IsString()
  @IsNotEmpty()
  document!: string;

  @ApiProperty()
  @IsString()
  @IsNotEmpty()
  documentType!: string;

  @ApiPropertyOptional()
  @IsString()
  @IsOptional()
  street?: string;

  @ApiPropertyOptional()
  @IsString()
  @IsOptional()
  number?: string;

  @ApiPropertyOptional()
  @IsString()
  @IsOptional()
  complement?: string;

  @ApiPropertyOptional()
  @IsString()
  @IsOptional()
  neighborhood?: string;

  @ApiPropertyOptional()
  @IsString()
  @IsOptional()
  city?: string;

  @ApiPropertyOptional()
  @IsString()
  @IsOptional()
  state?: string;

  @ApiPropertyOptional()
  @IsString()
  @IsOptional()
  zipCode?: string;

  @ApiPropertyOptional()
  @IsBoolean()
  @IsOptional()
  provisionNow?: boolean = true;
}
