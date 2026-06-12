import { ApiProperty } from '@nestjs/swagger';
import {
  IsNotEmpty,
  IsString,
  IsOptional,
  IsBoolean,
  IsDateString,
  IsEmail,
  IsNumber,
  IsInt,
} from 'class-validator';

export class CreateCompanyDto {
  @ApiProperty({
    example: 'uuid-revenda',
    description: 'ID da revenda (opcional para clientes diretos)',
    required: false,
  })
  @IsString()
  @IsOptional()
  revendaId?: string;

  @ApiProperty({
    example: 'uuid-client',
    description: 'ID do cliente (contratante)',
    required: false,
  })
  @IsString()
  @IsOptional()
  clientId?: string;

  @ApiProperty({ example: 'Empresa ABC', description: 'Nome da empresa' })
  @IsString()
  @IsNotEmpty()
  name!: string;

  @ApiProperty({
    example: 'empresa-abc',
    description:
      'Subdomínio da empresa (opcional, gerado automaticamente se vazio)',
    required: false,
  })
  @IsString()
  @IsOptional()
  subdomain?: string;

  @ApiProperty({
    example: 'empresa.com.br',
    description: 'Domínio customizado (opcional)',
    required: false,
  })
  @IsString()
  @IsOptional()
  customDomain?: string;

  @ApiProperty({
    example: true,
    description: 'Iniciar em modo demonstração (30 dias)',
    required: false,
    default: false,
  })
  @IsBoolean()
  @IsOptional()
  startInDemoMode?: boolean;

  @ApiProperty({
    example: true,
    description: 'Modo demonstração',
    required: false,
  })
  @IsBoolean()
  @IsOptional()
  isDemoMode?: boolean;

  @ApiProperty({
    example: true,
    description: 'Provisionar schema imediatamente após criação',
    required: false,
    default: true,
  })
  @IsBoolean()
  @IsOptional()
  provisionNow?: boolean;

  @ApiProperty({
    example: true,
    description: 'Empresa ativa',
    required: false,
    default: true,
  })
  @IsBoolean()
  @IsOptional()
  active?: boolean;

  @ApiProperty({
    example: 150.0,
    description: 'Valor da mensalidade',
    required: false,
  })
  @IsNumber()
  @IsOptional()
  monthlyFee?: number;

  @ApiProperty({
    example: 10,
    description: 'Dia de vencimento',
    required: false,
  })
  @IsInt()
  @IsOptional()
  billingDay?: number;

  @ApiProperty({
    example: 'BOLETO',
    description: 'Forma de pagamento',
    required: false,
  })
  @IsString()
  @IsOptional()
  paymentMethod?: string;

  @ApiProperty({
    example: '12.345.678/0001-90',
    description: 'Documento (CPF/CNPJ)',
    required: false,
  })
  @IsString()
  @IsOptional()
  document?: string;

  @ApiProperty({
    example: 'CNPJ',
    description: 'Tipo de documento (CPF, CNPJ, PASSPORT, etc)',
    required: false,
  })
  @IsString()
  @IsOptional()
  documentType?: string;

  @ApiProperty({
    example: 'empresa@email.com',
    description: 'Email de contato da empresa',
    required: false,
  })
  @IsEmail()
  @IsOptional()
  email?: string;

  @ApiProperty({
    example: '+55 11 99999-9999',
    description: 'Telefone de contato',
    required: false,
  })
  @IsString()
  @IsOptional()
  phone?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  taxRegime?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  accountantName?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  accountantEmail?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  stateRegistration?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  municipalRegistration?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  parentCompanyId?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  ownerUserId?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  zipCode?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  tradeName?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  segment?: string;

  @ApiProperty({ required: false })
  @IsDateString()
  @IsOptional()
  openingDate?: string;

  @ApiProperty({ required: false })
  @IsString()
  @IsOptional()
  openingHours?: string;

  @ApiProperty({ required: false })
  @IsOptional()
  modules?: string[];

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
}
