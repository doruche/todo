create table if not exists todos (
    id uuid primary key not null,
    title varchar (256) not null,
    description text,
    completed boolean not null default false
);