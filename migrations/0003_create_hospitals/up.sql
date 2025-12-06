CREATE TABLE hospitals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    address TEXT,
    phone VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
