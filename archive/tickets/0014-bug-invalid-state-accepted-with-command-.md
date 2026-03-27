commit 27e6688b5b8e718eb9504958c1c124d14167f13c
Author: philippepascal <philippe@pascalonline.org>
Date:   Thu Mar 26 17:06:01 2026 -0700

    ticket(14): ready → closed

diff --git a/tickets/0014-bug-invalid-state-accepted-with-command-.md b/tickets/0014-bug-invalid-state-accepted-with-command-.md
index 954e4f0..a277810 100644
--- a/tickets/0014-bug-invalid-state-accepted-with-command-.md
+++ b/tickets/0014-bug-invalid-state-accepted-with-command-.md
@@ -1,12 +1,11 @@
 +++
 id = 14
 title = "apm state accepts any string as state without validation"
-state = "ready"
+state = "closed"
 priority = 8
 effort = 2
 risk = 2
-created = "2026-03-25"
-updated = "2026-03-26"
+updated_at = "2026-03-27T00:06:00.986851Z"
 +++
 
 ## Spec
@@ -49,3 +48,4 @@ and return early before loading or modifying any ticket file.
 | 2026-03-25 | manual | new → specd | |
 | 2026-03-26 | manual | ammend → specd | |
 | 2026-03-26 | manual | specd → ready | |
+| 2026-03-27T00:06Z | ready | closed | apm |
