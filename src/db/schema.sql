-- DuckDB Schema Final

CREATE SEQUENCE IF NOT EXISTS users_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Users" (
    "id"            INTEGER   DEFAULT nextval('users_id_seq') PRIMARY KEY,
    "name"          VARCHAR   NOT NULL,
    "password_hash" VARCHAR   NOT NULL,
    "nickname"      VARCHAR   NOT NULL,
    "active"        BOOLEAN   NOT NULL DEFAULT TRUE,
    "updated_at"    TIMESTAMP NOT NULL,
    "created_at"    TIMESTAMP NOT NULL
);

CREATE SEQUENCE IF NOT EXISTS products_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Products" (
    "id"                 INTEGER       DEFAULT nextval('products_id_seq') PRIMARY KEY,
    "name"               VARCHAR       NOT NULL,
    "description"        VARCHAR,
    "active"             BOOLEAN       NOT NULL DEFAULT TRUE,
    "markup"             DECIMAL(5,4)  NOT NULL,
    "cost_price"         DECIMAL(10,2) NOT NULL DEFAULT 0,
    "final_price"        DECIMAL(10,2) NOT NULL,
    "expected_loss_rate" DECIMAL(5,4)  NOT NULL DEFAULT 0,
    "updated_at"         TIMESTAMP     NOT NULL,
    "created_at"         TIMESTAMP     NOT NULL
);

CREATE SEQUENCE IF NOT EXISTS product_components_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Product_Components" (
    "id"         INTEGER   DEFAULT nextval('product_components_id_seq') PRIMARY KEY,
    "parent_id"  INTEGER   NOT NULL,
    "child_id"   INTEGER   NOT NULL,
    "quantity"   INTEGER   NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("parent_id") REFERENCES "Products" ("id"),
    FOREIGN KEY ("child_id")  REFERENCES "Products" ("id"),
    CHECK ("parent_id" != "child_id")
);

CREATE SEQUENCE IF NOT EXISTS taxes_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Taxes" (
    "id"               INTEGER      DEFAULT nextval('taxes_id_seq') PRIMARY KEY,
    "name"             VARCHAR      NOT NULL,
    "application_type" VARCHAR      NOT NULL,
    "updated_at"       TIMESTAMP    NOT NULL,
    "created_at"       TIMESTAMP    NOT NULL
);

CREATE SEQUENCE IF NOT EXISTS product_taxes_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Product_Taxes" (
    "id"         INTEGER      DEFAULT nextval('product_taxes_id_seq') PRIMARY KEY,
    "product_id" INTEGER      NOT NULL,
    "tax_id"     INTEGER      NOT NULL,
    "rate"       DECIMAL(5,4) NOT NULL,
    "created_at" TIMESTAMP    NOT NULL,
    FOREIGN KEY ("product_id") REFERENCES "Products" ("id"),
    FOREIGN KEY ("tax_id")     REFERENCES "Taxes"    ("id")
);

CREATE SEQUENCE IF NOT EXISTS stock_movements_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Stock_Movements" (
    "id"            INTEGER   DEFAULT nextval('stock_movements_id_seq') PRIMARY KEY,
    "product_id"    INTEGER   NOT NULL,
    "quantity"      INTEGER   NOT NULL,
    "movement_type" VARCHAR   NOT NULL,
    "motive"        VARCHAR   NOT NULL,
    "created_at"    TIMESTAMP NOT NULL,
    FOREIGN KEY ("product_id") REFERENCES "Products" ("id")
);

CREATE VIEW IF NOT EXISTS "Stock" AS
    SELECT
        product_id,
        SUM(CASE
            WHEN movement_type = 'in'  THEN quantity
            WHEN movement_type = 'out' THEN -quantity
            ELSE 0
        END) AS current_quantity
    FROM "Stock_Movements"
    GROUP BY product_id;

CREATE SEQUENCE IF NOT EXISTS sales_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Sales" (
    "id"          INTEGER       DEFAULT nextval('sales_id_seq') PRIMARY KEY,
    "user_id"     INTEGER       NOT NULL,
    "total_value" DECIMAL(10,2) NOT NULL DEFAULT 0,
    "updated_at"  TIMESTAMP     NOT NULL,
    "created_at"  TIMESTAMP     NOT NULL,
    FOREIGN KEY ("user_id") REFERENCES "Users" ("id")
);

CREATE SEQUENCE IF NOT EXISTS sales_products_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Sales_Products" (
    "id"               INTEGER       DEFAULT nextval('sales_products_id_seq') PRIMARY KEY,
    "sale_id"          INTEGER       NOT NULL,
    "product_id"       INTEGER       NOT NULL,
    "quantity"         INTEGER       NOT NULL DEFAULT 1,
    "price"            DECIMAL(10,2) NOT NULL,
    "discount"         DECIMAL(5,4)  NOT NULL DEFAULT 0,
    "discounted_price" DECIMAL(10,2) NOT NULL DEFAULT 0,
    "created_at"       TIMESTAMP     NOT NULL,
    FOREIGN KEY ("sale_id")    REFERENCES "Sales"    ("id"),
    FOREIGN KEY ("product_id") REFERENCES "Products" ("id")
);

CREATE SEQUENCE IF NOT EXISTS payment_methods_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Payment_Methods" (
    "id"         INTEGER   DEFAULT nextval('payment_methods_id_seq') PRIMARY KEY,
    "name"       VARCHAR   NOT NULL,
    "active"     BOOLEAN   NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    "created_at" TIMESTAMP NOT NULL
);

CREATE SEQUENCE IF NOT EXISTS transaction_fees_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Transaction_Fees" (
    "id"                INTEGER       DEFAULT nextval('transaction_fees_id_seq') PRIMARY KEY,
    "payment_method_id" INTEGER       NOT NULL,
    "description"       VARCHAR       NOT NULL,
    "fee_percent"       DECIMAL(5,4)  NOT NULL,
    "fixed_fee"         DECIMAL(10,2) NOT NULL,
    "updated_at"        TIMESTAMP     NOT NULL,
    "created_at"        TIMESTAMP     NOT NULL,
    FOREIGN KEY ("payment_method_id") REFERENCES "Payment_Methods" ("id")
);

CREATE SEQUENCE IF NOT EXISTS sale_payments_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Sale_Payments" (
    "id"                INTEGER       DEFAULT nextval('sale_payments_id_seq') PRIMARY KEY,
    "sale_id"           INTEGER       NOT NULL,
    "payment_method_id" INTEGER       NOT NULL,
    "amount_paid"       DECIMAL(10,2) NOT NULL,
    "net_amount"        DECIMAL(10,2) NOT NULL,
    "created_at"        TIMESTAMP     NOT NULL,
    FOREIGN KEY ("sale_id")           REFERENCES "Sales"           ("id"),
    FOREIGN KEY ("payment_method_id") REFERENCES "Payment_Methods" ("id")
);

CREATE SEQUENCE IF NOT EXISTS financial_entries_id_seq START 1;
CREATE TABLE IF NOT EXISTS "Financial_Entries" (
    "id"           INTEGER       DEFAULT nextval('financial_entries_id_seq') PRIMARY KEY,
    "type"         VARCHAR       NOT NULL,
    "name"         VARCHAR       NOT NULL,
    "description"  VARCHAR,
    "category"     VARCHAR       NOT NULL,
    "amount"       DECIMAL(10,2) NOT NULL,
    "quantity"     INTEGER,
    "unit_cost"    DECIMAL(10,4),
    "product_id"   INTEGER,
    "due_date"     DATE,
    "is_recurring" BOOLEAN,
    "updated_at"   TIMESTAMP     NOT NULL,
    "created_at"   TIMESTAMP     NOT NULL,
    FOREIGN KEY ("product_id") REFERENCES "Products" ("id")
);
