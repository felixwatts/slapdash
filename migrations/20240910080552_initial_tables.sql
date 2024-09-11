create table series (
    id serial primary key,
    name text not null unique
);

create table point (
    id serial primary key,
    series_id int references series(id) not null,
    time timestamp without time zone not null,
    value real not null
);
