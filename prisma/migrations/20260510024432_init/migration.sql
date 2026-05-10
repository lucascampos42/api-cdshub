-- CreateEnum
CREATE TYPE "TicketStatus" AS ENUM ('AGUARDANDO_ATENDIMENTO', 'AGENDADO', 'EM_EXECUCAO', 'IMPLANTACAO', 'CONCLUIDO', 'CANCELADO');

-- CreateEnum
CREATE TYPE "TicketPriority" AS ENUM ('BAIXA', 'MEDIA', 'ALTA', 'URGENTE');

-- CreateEnum
CREATE TYPE "UserType" AS ENUM ('CODESDEVS_SUPERADMIN', 'CODESDEVS_SUPORTE', 'REVENDA_ADMIN', 'REVENDA_SUPORTE', 'REVENDA_GERENTE', 'REVENDA_CONTADOR', 'CLIENTE_ADMIN', 'CLIENTE_GERENTE', 'CLIENTE_FUNCIONARIO', 'CLIENTE_CONTADOR');

-- CreateEnum
CREATE TYPE "SuggestionStatus" AS ENUM ('ABERTO', 'EM_ANALISE', 'EM_DUPLICIDADE', 'EM_DESENVOLVIMENTO', 'CANCELADO', 'NAO_APROVADO', 'APROVADO', 'EM_TESTES', 'CONCLUIDO', 'DISPONIBILIZADO');

-- CreateTable
CREATE TABLE "access_rules" (
    "id" TEXT NOT NULL,
    "role" TEXT NOT NULL,
    "resource" TEXT NOT NULL,
    "can_read" BOOLEAN NOT NULL DEFAULT false,
    "can_write" BOOLEAN NOT NULL DEFAULT false,
    "can_update" BOOLEAN NOT NULL DEFAULT false,
    "can_delete" BOOLEAN NOT NULL DEFAULT false,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "access_rules_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "clients" (
    "id" TEXT NOT NULL,
    "revenda_id" TEXT,
    "name" TEXT NOT NULL,
    "document" TEXT,
    "document_type" TEXT,
    "email" TEXT,
    "phone" TEXT,
    "legal_rep_name" TEXT,
    "legal_rep_document" TEXT,
    "legal_rep_email" TEXT,
    "legal_rep_phone" TEXT,
    "zip_code" TEXT,
    "street" TEXT,
    "number" TEXT,
    "complement" TEXT,
    "neighborhood" TEXT,
    "city" TEXT,
    "state" TEXT,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "clients_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "companies" (
    "id" TEXT NOT NULL,
    "revenda_id" TEXT,
    "client_id" TEXT,
    "name" TEXT NOT NULL,
    "subdomain" TEXT NOT NULL,
    "custom_domain" TEXT,
    "active" BOOLEAN NOT NULL DEFAULT true,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,
    "demo_end_date" TIMESTAMP(3),
    "demo_start_date" TIMESTAMP(3),
    "is_demo_mode" BOOLEAN NOT NULL DEFAULT false,
    "billing_day" INTEGER,
    "city" TEXT,
    "complement" TEXT,
    "document" TEXT,
    "document_type" TEXT,
    "email" TEXT,
    "monthly_fee" DECIMAL(10,2),
    "neighborhood" TEXT,
    "number" TEXT,
    "payment_method" TEXT,
    "phone" TEXT,
    "segment" TEXT,
    "state" TEXT,
    "street" TEXT,
    "trade_name" TEXT,
    "zip_code" TEXT,
    "tax_regime" TEXT,
    "accountant_name" TEXT,
    "accountant_email" TEXT,
    "opening_date" DATE,
    "opening_hours" TEXT,
    "active_modules" TEXT[],
    "contract_number" TEXT,
    "contract_start_date" TIMESTAMP(3),
    "contract_end_date" TIMESTAMP(3),
    "contract_status" TEXT,
    "auto_renewal" BOOLEAN DEFAULT true,
    "notice_period_days" INTEGER,
    "signed_at" TIMESTAMP(3),
    "signed_by" TEXT,
    "billing_status" TEXT,
    "last_billing_date" TIMESTAMP(3),
    "next_billing_date" TIMESTAMP(3),
    "billing_suspended_at" TIMESTAMP(3),
    "late_fee_percent" DECIMAL(5,2),
    "interest_percent" DECIMAL(5,2),
    "gateway_customer_id" TEXT,
    "default_payment_method_id" TEXT,
    "payment_provider" TEXT,
    "issue_invoice" BOOLEAN DEFAULT true,
    "invoice_email" TEXT,
    "invoice_observations" TEXT,
    "cnae" TEXT,
    "municipal_tax_regime" TEXT,
    "simple_national_option" BOOLEAN,
    "simple_national_since" TIMESTAMP(3),
    "tax_incentives" JSONB,
    "nfse_settings" JSONB,
    "nfe_settings" JSONB,
    "cte_settings" JSONB,
    "nfce_settings" JSONB,
    "max_users" INTEGER,
    "max_branches" INTEGER,
    "max_invoices_per_month" INTEGER,
    "storage_limit_mb" INTEGER,
    "current_storage_usage_mb" INTEGER DEFAULT 0,
    "blocked_reason" TEXT,
    "blocked_at" TIMESTAMP(3),
    "last_access_at" TIMESTAMP(3),
    "last_login_ip" TEXT,
    "schema_name" TEXT,
    "db_connection_string" TEXT,
    "timezone" TEXT DEFAULT 'America/Sao_Paulo',
    "language" TEXT DEFAULT 'pt-BR',
    "dateFormat" TEXT DEFAULT 'dd/MM/yyyy',
    "numberFormat" TEXT DEFAULT 'pt-BR',
    "currency" TEXT DEFAULT 'BRL',
    "default_warehouse_id" TEXT,
    "default_price_table_id" TEXT,
    "support_email" TEXT,
    "support_phone" TEXT,
    "preferred_contact_method" TEXT,
    "notes" TEXT,
    "internal_tags" TEXT[],
    "lgpd_accepted_at" TIMESTAMP(3),
    "lgpd_version" TEXT,
    "data_retention_policy" TEXT,
    "two_factor_required" BOOLEAN DEFAULT false,
    "password_policy_id" TEXT,
    "state_registration" TEXT,
    "municipal_registration" TEXT,
    "sped_settings" JSONB,
    "parent_company_id" TEXT,
    "owner_user_id" TEXT,

    CONSTRAINT "companies_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "erp_instances" (
    "id" TEXT NOT NULL,
    "revenda_id" TEXT NOT NULL,
    "db_name" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "api_url" TEXT,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "erp_instances_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "logs" (
    "id" TEXT NOT NULL,
    "user_id" TEXT,
    "action" TEXT NOT NULL,
    "target" TEXT NOT NULL,
    "details" JSONB,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "logs_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "revendas" (
    "id" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "domain" TEXT NOT NULL,
    "active" BOOLEAN NOT NULL DEFAULT true,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,
    "city" TEXT,
    "complement" TEXT,
    "document" TEXT NOT NULL,
    "document_type" TEXT NOT NULL,
    "neighborhood" TEXT,
    "number" TEXT,
    "state" TEXT,
    "street" TEXT,
    "zip_code" TEXT,

    CONSTRAINT "revendas_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "contracts" (
    "id" TEXT NOT NULL,
    "revenda_id" TEXT,
    "company_id" TEXT,
    "title" TEXT NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'active',
    "start_date" TIMESTAMP(3) NOT NULL,
    "end_date" TIMESTAMP(3),
    "amount" DECIMAL(10,2),
    "file_url" TEXT,
    "terms" TEXT,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "contracts_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "users" (
    "id" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "email" TEXT NOT NULL,
    "password_hash" TEXT NOT NULL,
    "role" TEXT NOT NULL,
    "revenda_id" TEXT,
    "active" BOOLEAN NOT NULL DEFAULT true,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,
    "must_change_password" BOOLEAN NOT NULL DEFAULT true,
    "cpf" TEXT,
    "username" TEXT NOT NULL,
    "current_company_id" TEXT,
    "user_type" "UserType" NOT NULL DEFAULT 'CLIENTE_FUNCIONARIO',
    "hashed_refresh_token" TEXT,
    "two_factor_secret" TEXT,
    "is_two_factor_enabled" BOOLEAN NOT NULL DEFAULT false,

    CONSTRAINT "users_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "user_companies" (
    "id" TEXT NOT NULL,
    "user_id" TEXT NOT NULL,
    "company_id" TEXT NOT NULL,
    "role" TEXT NOT NULL,
    "is_default" BOOLEAN NOT NULL DEFAULT false,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "user_companies_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "tickets" (
    "id" TEXT NOT NULL,
    "revenda_id" TEXT NOT NULL,
    "company_id" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "status" "TicketStatus" NOT NULL DEFAULT 'AGUARDANDO_ATENDIMENTO',
    "priority" "TicketPriority" NOT NULL DEFAULT 'MEDIA',
    "category" TEXT,
    "created_by_id" TEXT NOT NULL,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,
    "closed_at" TIMESTAMP(3),
    "scheduled_for" TIMESTAMP(3),

    CONSTRAINT "tickets_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "ticket_assignments" (
    "id" TEXT NOT NULL,
    "ticket_id" TEXT NOT NULL,
    "user_id" TEXT NOT NULL,
    "is_primary" BOOLEAN NOT NULL DEFAULT false,
    "assigned_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "ticket_assignments_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "blog_posts" (
    "id" TEXT NOT NULL,
    "slug" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "content" TEXT NOT NULL,
    "cover_image" TEXT,
    "published" BOOLEAN NOT NULL DEFAULT false,
    "author" TEXT,
    "tags" TEXT[],
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "blog_posts_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "contact_requests" (
    "id" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "email" TEXT NOT NULL,
    "phone" TEXT,
    "subject" TEXT,
    "message" TEXT NOT NULL,
    "status" TEXT NOT NULL DEFAULT 'new',
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "contact_requests_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "suggestions" (
    "id" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "description" TEXT NOT NULL,
    "system" TEXT NOT NULL,
    "status" "SuggestionStatus" NOT NULL DEFAULT 'ABERTO',
    "votes" INTEGER NOT NULL DEFAULT 0,
    "created_by_id" TEXT,
    "created_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP(3) NOT NULL,

    CONSTRAINT "suggestions_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "access_rules_role_resource_key" ON "access_rules"("role", "resource");

-- CreateIndex
CREATE UNIQUE INDEX "clients_document_key" ON "clients"("document");

-- CreateIndex
CREATE INDEX "clients_revenda_id_idx" ON "clients"("revenda_id");

-- CreateIndex
CREATE UNIQUE INDEX "companies_document_key" ON "companies"("document");

-- CreateIndex
CREATE INDEX "companies_revenda_id_idx" ON "companies"("revenda_id");

-- CreateIndex
CREATE INDEX "companies_client_id_idx" ON "companies"("client_id");

-- CreateIndex
CREATE INDEX "companies_parent_company_id_idx" ON "companies"("parent_company_id");

-- CreateIndex
CREATE INDEX "companies_owner_user_id_idx" ON "companies"("owner_user_id");

-- CreateIndex
CREATE UNIQUE INDEX "companies_revenda_id_subdomain_key" ON "companies"("revenda_id", "subdomain");

-- CreateIndex
CREATE INDEX "erp_instances_revenda_id_idx" ON "erp_instances"("revenda_id");

-- CreateIndex
CREATE INDEX "logs_user_id_idx" ON "logs"("user_id");

-- CreateIndex
CREATE UNIQUE INDEX "revendas_domain_key" ON "revendas"("domain");

-- CreateIndex
CREATE UNIQUE INDEX "revendas_document_key" ON "revendas"("document");

-- CreateIndex
CREATE INDEX "contracts_revenda_id_idx" ON "contracts"("revenda_id");

-- CreateIndex
CREATE INDEX "contracts_company_id_idx" ON "contracts"("company_id");

-- CreateIndex
CREATE UNIQUE INDEX "users_email_key" ON "users"("email");

-- CreateIndex
CREATE UNIQUE INDEX "users_cpf_key" ON "users"("cpf");

-- CreateIndex
CREATE UNIQUE INDEX "users_username_key" ON "users"("username");

-- CreateIndex
CREATE INDEX "users_revenda_id_idx" ON "users"("revenda_id");

-- CreateIndex
CREATE INDEX "user_companies_user_id_idx" ON "user_companies"("user_id");

-- CreateIndex
CREATE INDEX "user_companies_company_id_idx" ON "user_companies"("company_id");

-- CreateIndex
CREATE UNIQUE INDEX "user_companies_user_id_company_id_key" ON "user_companies"("user_id", "company_id");

-- CreateIndex
CREATE INDEX "tickets_revenda_id_idx" ON "tickets"("revenda_id");

-- CreateIndex
CREATE INDEX "tickets_company_id_idx" ON "tickets"("company_id");

-- CreateIndex
CREATE INDEX "tickets_created_by_id_idx" ON "tickets"("created_by_id");

-- CreateIndex
CREATE INDEX "ticket_assignments_ticket_id_idx" ON "ticket_assignments"("ticket_id");

-- CreateIndex
CREATE INDEX "ticket_assignments_user_id_idx" ON "ticket_assignments"("user_id");

-- CreateIndex
CREATE UNIQUE INDEX "ticket_assignments_ticket_id_user_id_key" ON "ticket_assignments"("ticket_id", "user_id");

-- CreateIndex
CREATE UNIQUE INDEX "blog_posts_slug_key" ON "blog_posts"("slug");

-- AddForeignKey
ALTER TABLE "clients" ADD CONSTRAINT "clients_revenda_id_fkey" FOREIGN KEY ("revenda_id") REFERENCES "revendas"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "companies" ADD CONSTRAINT "companies_parent_company_id_fkey" FOREIGN KEY ("parent_company_id") REFERENCES "companies"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "companies" ADD CONSTRAINT "companies_owner_user_id_fkey" FOREIGN KEY ("owner_user_id") REFERENCES "users"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "companies" ADD CONSTRAINT "companies_revenda_id_fkey" FOREIGN KEY ("revenda_id") REFERENCES "revendas"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "companies" ADD CONSTRAINT "companies_client_id_fkey" FOREIGN KEY ("client_id") REFERENCES "clients"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "erp_instances" ADD CONSTRAINT "erp_instances_revenda_id_fkey" FOREIGN KEY ("revenda_id") REFERENCES "revendas"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "logs" ADD CONSTRAINT "logs_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "contracts" ADD CONSTRAINT "contracts_revenda_id_fkey" FOREIGN KEY ("revenda_id") REFERENCES "revendas"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "contracts" ADD CONSTRAINT "contracts_company_id_fkey" FOREIGN KEY ("company_id") REFERENCES "companies"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "users" ADD CONSTRAINT "users_revenda_id_fkey" FOREIGN KEY ("revenda_id") REFERENCES "revendas"("id") ON DELETE SET NULL ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "user_companies" ADD CONSTRAINT "user_companies_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "user_companies" ADD CONSTRAINT "user_companies_company_id_fkey" FOREIGN KEY ("company_id") REFERENCES "companies"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "tickets" ADD CONSTRAINT "tickets_revenda_id_fkey" FOREIGN KEY ("revenda_id") REFERENCES "revendas"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "tickets" ADD CONSTRAINT "tickets_company_id_fkey" FOREIGN KEY ("company_id") REFERENCES "companies"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "tickets" ADD CONSTRAINT "tickets_created_by_id_fkey" FOREIGN KEY ("created_by_id") REFERENCES "users"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "ticket_assignments" ADD CONSTRAINT "ticket_assignments_ticket_id_fkey" FOREIGN KEY ("ticket_id") REFERENCES "tickets"("id") ON DELETE CASCADE ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "ticket_assignments" ADD CONSTRAINT "ticket_assignments_user_id_fkey" FOREIGN KEY ("user_id") REFERENCES "users"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "suggestions" ADD CONSTRAINT "suggestions_created_by_id_fkey" FOREIGN KEY ("created_by_id") REFERENCES "users"("id") ON DELETE SET NULL ON UPDATE CASCADE;
