import { IsEmail, IsNotEmpty, IsOptional, IsString } from 'class-validator';
import { ApiProperty } from '@nestjs/swagger';

export class CreateUserDto {
  @ApiProperty({ example: 'John Doe', description: 'Nome completo do usuário' })
  @IsString()
  @IsNotEmpty()
  name!: string;

  @ApiProperty({
    example: 'johndoe',
    description: 'Nome de usuário/apelido único para login',
  })
  @IsString()
  @IsNotEmpty()
  username!: string;

  @ApiProperty({
    example: 'john@example.com',
    description: 'E-mail único do usuário',
  })
  @IsEmail()
  @IsNotEmpty()
  email!: string;

  @ApiProperty({
    example: '12345678900',
    description: 'CPF do usuário (opcional)',
    required: false,
  })
  @IsString()
  @IsOptional()
  cpf?: string;

  @ApiProperty({
    example: 'admin',
    description: 'Role do usuário (admin, operador, revendedor)',
  })
  @IsString()
  @IsNotEmpty()
  role!: string;

  @ApiProperty({
    example: 'uuid-revenda',
    description: 'ID da revenda (opcional)',
    required: false,
  })
  @IsString()
  @IsOptional()
  revendaId?: string;
}
