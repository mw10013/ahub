create table if not exists AccessHub (
    id integer not null primary key,
    cloud_last_access_event_at datetime
);
create table if not exists AccessPoint (
    id integer not null primary key,
    position integer not null
);
create table if not exists AccessUser (
    id integer not null primary key,
    name text not null default '',
    code text not null,
    activate_code_at datetime,
    expire_code_at datetime
);
create table if not exists AccessEvent (
    id integer not null primary key,
    at datetime not null,
    access text not null,
    code text not null,
    access_user_id integer,
    access_point_id integer not null,
    constraint AccessEvent_access_point_id_fkey foreign key (access_point_id) references AccessPoint (id) on delete restrict on update cascade
);
create table if not exists AccessPointToAccessUser (
    access_point_id integer not null,
    access_user_id integer not null,
    foreign key (access_point_id) references AccessPoint (id) on delete cascade on update cascade,
    foreign key (access_user_id) references AccessUser (id) on delete cascade on update cascade
);

create view if not exists ActiveCode as 
select access_point_id, position, code, access_user_id, activate_code_at, expire_code_at 
from AccessUser u join AccessPointToAccessUser p2u on u.id = p2u.access_user_id 
  join AccessPoint p on p2u.access_point_id = p.id 
where (activate_code_at is null or activate_code_at <= current_timestamp) 
  and (expire_code_at is null or current_timestamp < expire_code_at) 
order by position asc, code asc;

create unique index AccessPoint_position_key on AccessPoint(position);
create unique index AccessUser_code_key on AccessUser(code);
create unique index AccessPointToAccessUser_unique on AccessPointToAccessUser(access_point_id, access_user_id);
create index AccessPointToAccessUser_access_user_id_index on AccessPointToAccessUser(access_user_id);
