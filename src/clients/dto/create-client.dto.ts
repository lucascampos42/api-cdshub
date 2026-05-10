import { ApiProperty } from '@nestjs/swagger';
import { IsNotEmpty, IsString, IsOptional, IsEmail } from 'class-validator';

export class CreateClientDto {
  @ApiProperty({ description: 'ID da revenda', required: false })
  @IsString()
  @IsOptional()
  revendaId?: string;

  @ApiProperty({ description: 'Nome do cliente' })
  @IsString()
  @IsNotEmpty()
  name: string;

  @ApiProperty({ description: 'CPF/CNPJ', required: false })
  @IsString()
  @IsOptional()
  document?: string;

  @ApiProperty({ description: 'Tipo de documento', required: false })
  @IsString()
  @IsOptional()
  documentType?: string;

  @ApiProperty({ description: 'Email principal', required: false })
  @IsEmail()
  @IsOptional()
  email?: string;

  @ApiProperty({ description: 'Telefone', required: false })
  @IsString()
  @IsOptional()
  phone?: string;

  // Responsável Legal
  @ApiProperty({ description: 'Nome do responsável legal', required: false })
  @IsString()
  @IsOptional()
  legalRepresentativeName?: string;

  @ApiProperty({
    description: 'Documento do responsável legal',
    required: false,
  })
  @IsString()
  @IsOptional()
  legalRepresentativeDocument?: string;

  @ApiProperty({ description: 'Email do responsável legal', required: false })
  @IsString()
  @IsOptional()
  legalRepresentativeEmail?: string;

  @ApiProperty({
    description: 'Telefone do responsável legal',
    required: false,
  })
  @IsString()
  @IsOptional()
  legalRepresentativePhone?: string;

  // Endereço
  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  zipCode?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  street?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  number?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  complement?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  neighborhood?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  city?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  state?: string;

  @ApiProperty({ description: 'IDs dos sistemas contratados', type: [String], required: false })
  @IsOptional()
  systemIds?: string[];
}
