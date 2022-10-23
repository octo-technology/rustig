CREATE TABLE "objects" (
  "id" TEXT PRIMARY KEY,
  "type" TEXT NOT NULL CHECK("type" IN ('blob','tree')),
  "data" TEXT NOT NULL
) STRICT;

CREATE INDEX "objects_id" ON "objects"("id");
