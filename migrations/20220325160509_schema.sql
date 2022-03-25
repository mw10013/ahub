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
    "activateCodeAt" DATETIME,
    "expireCodeAt" DATETIME
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
CREATE TABLE IF NOT EXISTS "_AccessPointToAccessUser" (
    "A" INTEGER NOT NULL,
    "B" INTEGER NOT NULL,
    FOREIGN KEY ("A") REFERENCES "AccessPoint" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY ("B") REFERENCES "AccessUser" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
CREATE UNIQUE INDEX "AccessPoint_position_key" ON "AccessPoint"("position");
CREATE UNIQUE INDEX "AccessUser_code_key" ON "AccessUser"("code");
CREATE UNIQUE INDEX "_AccessPointToAccessUser_AB_unique" ON "_AccessPointToAccessUser"("A", "B");
CREATE INDEX "_AccessPointToAccessUser_B_index" ON "_AccessPointToAccessUser"("B");
