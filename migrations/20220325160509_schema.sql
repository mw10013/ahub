CREATE TABLE IF NOT EXISTS "AccessHub" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL DEFAULT '',
    "cloudLastAccessEventAt" DATETIME
);
/* CREATE TABLE sqlite_sequence(name,seq); */
CREATE TABLE IF NOT EXISTS "AccessPoint" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL DEFAULT '',
    "accessHubId" INTEGER NOT NULL,
    "position" INTEGER NOT NULL,
    CONSTRAINT "AccessPoint_accessHubId_fkey" FOREIGN KEY ("accessHubId") REFERENCES "AccessHub" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);
CREATE TABLE IF NOT EXISTS "AccessUser" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL DEFAULT '',
    "code" TEXT NOT NULL,
    "activateCodeAt" DATETIME,
    "expireCodeAt" DATETIME,
    "accessHubId" INTEGER NOT NULL,
    CONSTRAINT "AccessUser_accessHubId_fkey" FOREIGN KEY ("accessHubId") REFERENCES "AccessHub" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);
CREATE TABLE IF NOT EXISTS "AccessEvent" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "at" DATETIME NOT NULL,
    "access" TEXT NOT NULL,
    "code" TEXT NOT NULL,
    "accessUserId" INTEGER,
    "accessPointId" INTEGER NOT NULL,
    CONSTRAINT "AccessEvent_accessPointId_fkey" FOREIGN KEY ("accessPointId") REFERENCES "AccessPoint" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);
CREATE TABLE IF NOT EXISTS "_AccessPointToAccessUser" (
    "A" INTEGER NOT NULL,
    "B" INTEGER NOT NULL,
    FOREIGN KEY ("A") REFERENCES "AccessPoint" ("id") ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY ("B") REFERENCES "AccessUser" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
CREATE UNIQUE INDEX "AccessPoint_accessHubId_position_key" ON "AccessPoint"("accessHubId", "position");
CREATE UNIQUE INDEX "AccessUser_code_key" ON "AccessUser"("code");
CREATE UNIQUE INDEX "_AccessPointToAccessUser_AB_unique" ON "_AccessPointToAccessUser"("A", "B");
CREATE INDEX "_AccessPointToAccessUser_B_index" ON "_AccessPointToAccessUser"("B");
