# T-1884 S3 — CLI-watch render validator (smoke)

Targets: 1 (smoke — one representative)

Technique: script -c capture + split on \x1b[2J\x1b[H + normalize timestamps + frame-body diff

## T-1486 — agent presence --watch view is steady

   cmd:      `termlink agent presence --watch --watch-interval 2`
   duration: 8s
   interval: 2s
   expected frames: ~4

   exit:     0
   raw size: 750 bytes
   frames:   4
   verdict:  **PASS-LOOSE** (4 frames, 2 distinct bodies (content changed in-window, no flicker pattern))

   first-frame preview (normalized):

     | # agent presence --watch | view=by-peer | interval=2s | window=3600s | <TS>
     | (no peers active in window=3600s)

