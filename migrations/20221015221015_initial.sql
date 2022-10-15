CREATE TABLE "objects" (
  "id" TEXT PRIMARY KEY,
  "type" TEXT NOT NULL CHECK("type" IN ('blob','tree')),
  "data" BLOB NOT NULL
);

CREATE INDEX "objects_id" ON "objects"("id");
