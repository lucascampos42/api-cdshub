import { PrismaClient } from '@prisma/client';
import * as bcrypt from 'bcrypt';
import * as dotenv from 'dotenv';
import * as path from 'path';
import { Pool } from 'pg';
import { PrismaPg } from '@prisma/adapter-pg';

dotenv.config({ path: path.join(__dirname, '../.env') });

const connectionString = process.env.DATABASE_URL;
const pool = new Pool({ connectionString });
const adapter = new PrismaPg(pool);

const prisma = new PrismaClient({
  adapter,
  log: ['info', 'error'],
});

async function main() {
  console.log('🌱 Iniciando seed do api-flow...');

  const hashedPassword = await bcrypt.hash('12345678', 10);

  const email = 'bhlucascampos@gmail.com';
  const username = 'bhlucascampos';

  const existingUser = await prisma.user.findFirst({
    where: {
      OR: [
        { email: email },
        { username: username }
      ]
    },
  });

  let masterUser;

  if (existingUser) {
    // Atualizar usuário existente para garantir que é SuperAdmin
    masterUser = await prisma.user.update({
      where: { id: existingUser.id },
      data: {
        name: 'Lucas Campos (CodesDevs)',
        email: email,
        username: username,
        passwordHash: hashedPassword,
        userType: 'CODESDEVS_SUPERADMIN' as any,
        role: 'superadmin',
        active: true,
        mustChangePassword: false,
      },
    });
    console.log('✅ Usuário Master CodesDevs atualizado');
  } else {
    // Criar novo usuário Master
    masterUser = await prisma.user.create({
      data: {
        name: 'Lucas Campos (CodesDevs)',
        email: email,
        username: username,
        passwordHash: hashedPassword,
        userType: 'CODESDEVS_SUPERADMIN' as any,
        role: 'superadmin',
        active: true,
        mustChangePassword: false,
      },
    });
    console.log('✅ Usuário Master CodesDevs criado');
  }

  console.log('');
  console.log('📋 Dados do usuário master:');
  console.log(`   ID: ${masterUser.id}`);
  console.log(`   Email: ${masterUser.email}`);
  console.log(`   Username: ${masterUser.username}`);
  console.log(`   Tipo: ${masterUser.userType}`);
  console.log('');
  console.log('🔑 Senha: 12345678');
  console.log('');
}

main()
  .catch((e) => {
    console.error('❌ Erro ao executar seed:', e);
    process.exit(1);
  })
  .finally(async () => {
    await prisma.$disconnect();
  });
