commit fff2daa873a306ddf6fa731c58250d6842f6cacc
Author: philippepascal <philippe@pascalonline.org>
Date:   Thu Mar 26 17:06:01 2026 -0700

    ticket(17): ready → closed

diff --git a/tickets/0017-apm-sync-missing-quiet-and-offline-flags.md b/tickets/0017-apm-sync-missing-quiet-and-offline-flags.md
index a1bf2d9..12e3669 100644
--- a/tickets/0017-apm-sync-missing-quiet-and-offline-flags.md
+++ b/tickets/0017-apm-sync-missing-quiet-and-offline-flags.md
@@ -1,13 +1,12 @@
 +++
 id = 17
 title = "apm sync missing --quiet and --offline flags"
-state = "ready"
+state = "closed"
 priority = 10
 effort = 1
 risk = 1
 branch = "ticket/0017-apm-sync-missing-quiet-and-offline-flags"
-created = "2026-03-26"
-updated = "2026-03-26"
+updated_at = "2026-03-27T00:06:01.145860Z"
 +++
 
 ## Spec
@@ -42,3 +41,4 @@ and gate all `println!` calls behind `!quiet`.
 |------|-------|------------|------|
 | 2026-03-26 | manual | new → specd | |
 | 2026-03-26 | manual | specd → ready | |
+| 2026-03-27T00:06Z | ready | closed | apm |
