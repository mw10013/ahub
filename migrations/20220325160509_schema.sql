CREATE TABLE IF NOT EXISTS "AccessHub" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "cloud_last_access_event_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "AccessPoint" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "position" INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS "AccessUser" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "name" TEXT NOT NULL DEFAULT '',
    "code" TEXT NOT NULL,
    "activate_code_at" DATETIME,
    "expire_code_at" DATETIME
);
CREATE TABLE IF NOT EXISTS "AccessEvent" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "at" DATETIME NOT NULL,
    "access" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "access_user_id" INTEGER,
    "access_point_id" INTEGER NOT NULL,
    CONSTRAINT "AccessEvent_access_point_id_fkey" FOREIGN KEY ("access_point_id") REFERENCES "AccessPoint" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);
CREATE TABLE IF NOT EXISTS "AccessPointToAccessUser" (
    "access_point_id" INTEGER NOT NULL,
    "access_user_id" INTEGER NOT NULL,
    FOREIGN KEY ("access_point_id") REFERENCES "AccessPoint" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY ("access_user_id") REFERENCES "AccessUser" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

create view if not exists ActiveCodes as 
select access_point_id, position, code, access_user_id, activate_code_at, expire_code_at 
from AccessUser u join AccessPointToAccessUser p2u on u.id = p2u.access_user_id 
  join AccessPoint p on p2u.access_point_id = p.id 
where (activate_code_at is null or activate_code_at <= current_timestamp) 
  and (expire_code_at is null or current_timestamp < expire_code_at) 
order by position asc, code asc;

CREATE UNIQUE INDEX "AccessPoint_position_key" ON "AccessPoint"("position");
CREATE UNIQUE INDEX "AccessUser_code_key" ON "AccessUser"("code");
CREATE UNIQUE INDEX "AccessPointToAccessUser_unique" ON "AccessPointToAccessUser"("access_point_id", "access_user_id");
CREATE INDEX "AccessPointToAccessUser_access_user_id_index" ON "AccessPointToAccessUser"("access_user_id");
