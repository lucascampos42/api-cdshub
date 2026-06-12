-- CreateTable
CREATE TABLE "systems" (
    "id" TEXT NOT NULL,
    "slug" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT,
    "active" BOOLEAN NOT NULL DEFAULT true,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "systems_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "revenda_systems" (
    "id" TEXT NOT NULL,
    "revenda_id" TEXT NOT NULL,
    "system_id" TEXT NOT NULL,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "revenda_systems_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "company_systems" (
    "id" TEXT NOT NULL,
    "company_id" TEXT NOT NULL,
    "system_id" TEXT NOT NULL,
    "active" BOOLEAN NOT NULL DEFAULT true,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "company_systems_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "systems_slug_key" ON "systems"("slug");

-- CreateIndex
CREATE UNIQUE INDEX "revenda_systems_revenda_id_system_id_key" ON "revenda_systems"("revenda_id", "system_id");

-- CreateIndex
CREATE UNIQUE INDEX "company_systems_company_id_system_id_key" ON "company_systems"("company_id", "system_id");

-- AddForeignKey
ALTER TABLE "revenda_systems" ADD CONSTRAINT "revenda_systems_revenda_id_fkey" FOREIGN KEY ("revenda_id") REFERENCES "revendas"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "revenda_systems" ADD CONSTRAINT "revenda_systems_system_id_fkey" FOREIGN KEY ("system_id") REFERENCES "systems"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "company_systems" ADD CONSTRAINT "company_systems_company_id_fkey" FOREIGN KEY ("company_id") REFERENCES "companies"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "company_systems" ADD CONSTRAINT "company_systems_system_id_fkey" FOREIGN KEY ("system_id") REFERENCES "systems"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
