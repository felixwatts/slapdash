create table series (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name text not null unique
);

create table point (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    series_id INTEGER references series(id) not null,
    time INTEGER not null,
    value REAL not null
);
