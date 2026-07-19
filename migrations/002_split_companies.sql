-- Split companies table into sub-tables following single responsibility

-- 1. Company addresses
CREATE TABLE company_addresses (
    company_id    TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    street        TEXT,
    number        TEXT,
    complement    TEXT,
    neighborhood  TEXT,
    city          TEXT,
    state         TEXT,
    zip_code      TEXT,
    created_at    TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO company_addresses (company_id, street, number, complement, neighborhood, city, state, zip_code, created_at, updated_at)
SELECT id, street, number, complement, neighborhood, city, state, zip_code, created_at, updated_at
FROM companies WHERE street IS NOT NULL OR number IS NOT NULL OR complement IS NOT NULL OR neighborhood IS NOT NULL OR city IS NOT NULL OR state IS NOT NULL OR zip_code IS NOT NULL;

-- 2. Company contacts
CREATE TABLE company_contacts (
    company_id               TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    email                    TEXT,
    phone                    TEXT,
    support_email            TEXT,
    support_phone            TEXT,
    preferred_contact_method TEXT,
    created_at               TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at               TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO company_contacts (company_id, email, phone, support_email, support_phone, preferred_contact_method, created_at, updated_at)
SELECT id, email, phone, support_email, support_phone, preferred_contact_method, created_at, updated_at
FROM companies WHERE email IS NOT NULL OR phone IS NOT NULL;

-- 3. Company tax info
CREATE TABLE company_tax_info (
    company_id             TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    document               TEXT,
    document_type          TEXT,
    trade_name             TEXT,
    state_registration     TEXT,
    municipal_registration TEXT,
    cnae                   TEXT,
    tax_regime             TEXT,
    municipal_tax_regime   TEXT,
    simple_national_option BOOLEAN,
    simple_national_since  TIMESTAMP,
    tax_incentives         JSONB,
    created_at             TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO company_tax_info (company_id, document, document_type, trade_name, state_registration, municipal_registration, cnae, tax_regime, municipal_tax_regime, simple_national_option, simple_national_since, tax_incentives, created_at, updated_at)
SELECT id, document, document_type, trade_name, state_registration, municipal_registration, cnae, tax_regime, municipal_tax_regime, simple_national_option, simple_national_since, tax_incentives, created_at, updated_at
FROM companies WHERE document IS NOT NULL;

-- 4. Company billing
CREATE TABLE company_billing (
    company_id                TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    billing_day               INTEGER,
    monthly_fee               DECIMAL(10,2),
    payment_method            TEXT,
    billing_status            TEXT,
    last_billing_date         TIMESTAMP,
    next_billing_date         TIMESTAMP,
    billing_suspended_at      TIMESTAMP,
    late_fee_percent          DECIMAL(5,2),
    interest_percent          DECIMAL(5,2),
    gateway_customer_id       TEXT,
    default_payment_method_id TEXT,
    payment_provider          TEXT,
    issue_invoice             BOOLEAN,
    invoice_email             TEXT,
    invoice_observations      TEXT,
    created_at                TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO company_billing (company_id, billing_day, monthly_fee, payment_method, billing_status, last_billing_date, next_billing_date, billing_suspended_at, late_fee_percent, interest_percent, gateway_customer_id, default_payment_method_id, payment_provider, issue_invoice, invoice_email, invoice_observations, created_at, updated_at)
SELECT id, billing_day, monthly_fee, payment_method, billing_status, last_billing_date, next_billing_date, billing_suspended_at, late_fee_percent, interest_percent, gateway_customer_id, default_payment_method_id, payment_provider, issue_invoice, invoice_email, invoice_observations, created_at, updated_at
FROM companies WHERE monthly_fee IS NOT NULL OR billing_day IS NOT NULL;

-- 5. Company contracts
CREATE TABLE company_contracts (
    company_id           TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    contract_number      TEXT,
    contract_start_date  TIMESTAMP,
    contract_end_date    TIMESTAMP,
    contract_status      TEXT,
    auto_renewal         BOOLEAN,
    notice_period_days   INTEGER,
    signed_at            TIMESTAMP,
    signed_by            TEXT,
    created_at           TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO company_contracts (company_id, contract_number, contract_start_date, contract_end_date, contract_status, auto_renewal, notice_period_days, signed_at, signed_by, created_at, updated_at)
SELECT id, contract_number, contract_start_date, contract_end_date, contract_status, auto_renewal, notice_period_days, signed_at, signed_by, created_at, updated_at
FROM companies WHERE contract_number IS NOT NULL;

-- 6. Company plan limits
CREATE TABLE company_plan_limits (
    company_id                TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    max_users                 INTEGER,
    max_branches              INTEGER,
    max_invoices_per_month    INTEGER,
    storage_limit_mb          INTEGER,
    current_storage_usage_mb  INTEGER,
    created_at                TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at                TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO company_plan_limits (company_id, max_users, max_branches, max_invoices_per_month, storage_limit_mb, current_storage_usage_mb, created_at, updated_at)
SELECT id, max_users, max_branches, max_invoices_per_month, storage_limit_mb, current_storage_usage_mb, created_at, updated_at
FROM companies WHERE max_users IS NOT NULL;

-- 7. Company settings
CREATE TABLE company_settings (
    company_id             TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    timezone               TEXT,
    language               TEXT,
    date_format            TEXT,
    number_format          TEXT,
    currency               TEXT,
    default_warehouse_id   TEXT,
    default_price_table_id TEXT,
    opening_hours          TEXT,
    opening_date           DATE,
    segment                TEXT,
    active_modules         TEXT[],
    systems                TEXT[],
    notes                  TEXT,
    internal_tags          TEXT[],
    created_at             TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at             TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO company_settings (company_id, timezone, language, date_format, number_format, currency, default_warehouse_id, default_price_table_id, opening_hours, opening_date, segment, active_modules, systems, notes, internal_tags, created_at, updated_at)
SELECT id, timezone, language, date_format, number_format, currency, default_warehouse_id, default_price_table_id, opening_hours, opening_date, segment, active_modules, systems, notes, internal_tags, created_at, updated_at
FROM companies WHERE timezone IS NOT NULL;

-- 8. Company security
CREATE TABLE company_security (
    company_id            TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    lgpd_accepted_at      TIMESTAMP,
    lgpd_version          TEXT,
    data_retention_policy TEXT,
    two_factor_required   BOOLEAN,
    password_policy_id    TEXT,
    created_at            TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at            TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO company_security (company_id, lgpd_accepted_at, lgpd_version, data_retention_policy, two_factor_required, password_policy_id, created_at, updated_at)
SELECT id, lgpd_accepted_at, lgpd_version, data_retention_policy, two_factor_required, password_policy_id, created_at, updated_at
FROM companies WHERE lgpd_accepted_at IS NOT NULL;

-- 9. Company activity
CREATE TABLE company_activity (
    company_id      TEXT PRIMARY KEY REFERENCES companies(id) ON DELETE CASCADE,
    last_access_at  TIMESTAMP,
    last_login_ip   TEXT,
    blocked_reason  TEXT,
    blocked_at      TIMESTAMP,
    created_at      TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP NOT NULL DEFAULT NOW()
);

INSERT INTO company_activity (company_id, last_access_at, last_login_ip, blocked_reason, blocked_at, created_at, updated_at)
SELECT id, last_access_at, last_login_ip, blocked_reason, blocked_at, created_at, updated_at
FROM companies WHERE last_access_at IS NOT NULL;

-- After verifying data, drop columns from companies:
-- ALTER TABLE companies DROP COLUMN street, DROP COLUMN number, DROP COLUMN complement, DROP COLUMN neighborhood, DROP COLUMN city, DROP COLUMN state, DROP COLUMN zip_code;
-- ALTER TABLE companies DROP COLUMN email, DROP COLUMN phone, DROP COLUMN support_email, DROP COLUMN support_phone, DROP COLUMN preferred_contact_method;
-- ... etc for all extracted columns
