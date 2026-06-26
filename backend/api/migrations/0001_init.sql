-- Initial schema. The backend stores the family graph, pairing, policies, and
-- a per-recipient queue of OPAQUE alert envelopes. There is no column for alert
-- content: the envelope is ciphertext bytes the backend never decrypts.

create table if not exists users (
    id            text primary key,
    email         text unique not null,
    password_hash text not null,
    created_at    timestamptz not null default now()
);

create table if not exists children (
    id         text primary key,
    parent_id  text not null references users(id),
    created_at timestamptz not null default now()
);

create table if not exists pairing_codes (
    code         text primary key,
    child_id     text not null references children(id),
    recipient_id text not null,
    expires_at   timestamptz not null,
    used         boolean not null default false
);

create table if not exists devices (
    child_device_id text primary key,
    child_id        text not null references children(id),
    recipient_id    text not null,
    platform        text not null,
    public_key      text not null default '',
    capabilities    text[] not null default '{}'
);

create table if not exists policies (
    child_device_id text primary key references devices(child_device_id),
    policy          jsonb not null,
    updated_at      timestamptz not null default now()
);

create table if not exists alert_envelopes (
    id           bigserial primary key,
    recipient_id text not null,
    envelope     bytea not null,
    created_at   timestamptz not null default now()
);

create index if not exists alert_envelopes_recipient_idx
    on alert_envelopes (recipient_id, id);
