# E33: MARS as an agent's episodic memory (incident response), noise 0

Config: {"seed": 1, "noise": 0, "templates": 40, "novel": 10, "incidents": 600, "seed_episodes": 60, "k": 5, "feedback_top": 3, "hidden_root_rate": 0.5}

| memory | fix@1 (learned) | fix@1 (raw) | fix@3 | root observed | root hidden | novel mechanism, first occurrence | top analogue = same mechanism | fix@1 by stream quarter |
|---|---|---|---|---|---|---|---|---|
| MARS (structure) | 0.995 | 0.995 | 0.997 | 1.000 | 0.990 | 0.40 (10) | 1.000 | 0.99 / 0.99 / 1.00 / 0.99 |
| MARS + names (identity 0.3) | 0.990 | 0.993 | 0.997 | 1.000 | 0.980 | 0.40 (10) | 0.992 | 0.97 / 1.00 / 1.00 / 0.99 |
| recall by names (identity only) | 0.475 | 0.342 | 0.554 | 0.581 | 0.367 | 0.10 (10) | 0.166 | 0.37 / 0.46 / 0.52 / 0.55 |
| popularity baseline | 0.086 | 0.086 | 0.086 | 0.084 | 0.088 | 0.00 (10) | — | 0.09 / 0.10 / 0.09 / 0.07 |

**Abstention by significance** (accept the top suggestion when z ≥ T):

| memory | T | coverage (known mechanisms) | precision of accepted | first occurrences of novel mechanisms abstained |
|---|---|---|---|---|
| MARS (structure) | 3.0 | 1.000 | 0.995 | 0.00 |
| MARS (structure) | 5.0 | 0.998 | 0.995 | 0.10 |
| MARS (structure) | 9.0 | 0.969 | 0.995 | 0.80 |
| MARS + names (identity 0.3) | 3.0 | 1.000 | 0.990 | 0.00 |
| MARS + names (identity 0.3) | 5.0 | 0.946 | 0.995 | 0.60 |
| MARS + names (identity 0.3) | 9.0 | 0.529 | 1.000 | 1.00 |
| recall by names (identity only) | 3.0 | 0.261 | 0.578 | 0.40 |
| recall by names (identity only) | 5.0 | 0.014 | 0.250 | 1.00 |
| recall by names (identity only) | 9.0 | 0.002 | 0.000 | 1.00 |

**Rules the memory induced** (MARS, ≥ 10 feedback outcomes, precision ≥ 0.5):

| rule | precision | outcomes |
|---|---|---|
| `remedy-reissue-cert: unlinked` | 1.000 | 37 |
| `remedy-warm-cache: unlinked` | 1.000 | 35 |
| `remedy-dead-letter: unlinked` | 1.000 | 20 |
| `remedy-resync-clock: unlinked` | 1.000 | 18 |
| `remedy-resize-cache(x, y) ⇐ alerted-on(x, z) ∧ reads-cache(z, y)` | 1.000 | 17 |
| `remedy-resize-cache: unlinked` | 1.000 | 17 |
| `remedy-kill-blocking-query(x, y) ⇐ alerted-on(x, z) ∧ uses-db(z, y)` | 1.000 | 15 |
| `remedy-failover-db(x, y) ⇐ alerted-on(x, z) ∧ uses-db(z, y)` | 1.000 | 13 |
| `remedy-failover-db: unlinked` | 1.000 | 12 |
| `remedy-warm-cache(x, y) ⇐ alerted-on(x, z) ∧ reads-cache(z, y)` | 1.000 | 11 |
| `remedy-add-consumers: unlinked` | 0.981 | 53 |
| `remedy-reroute-traffic(x, y) ⇐ alerted-on(x, z) ∧ routes-via(z, y)` | 0.976 | 42 |
| `remedy-dead-letter(x, y) ⇐ alerted-on(x, z) ∧ consumes(z, y)` | 0.976 | 41 |
| `remedy-replace-disk: unlinked` | 0.955 | 22 |
| `remedy-add-consumers(x, y) ⇐ alerted-on(x, z) ∧ consumes(z, y)` | 0.950 | 20 |
| `remedy-reboot-host(x, y) ⇐ alerted-on(x, z) ∧ runs-on(z, y)` | 0.926 | 27 |
| `remedy-renew-cert(x, y) ⇐ alerted-on(x, z) ∧ serves-cert(z, y)` | 0.917 | 12 |
| `remedy-resync-clock(x, y) ⇐ alerted-on(x, z) ∧ runs-on(z, y)` | 0.912 | 57 |
| `remedy-replace-disk(x, y) ⇐ alerted-on(x, z) ∧ writes-to(z, y)` | 0.892 | 37 |
| `remedy-fix-dns(x, y) ⇐ alerted-on(x, z) ∧ routes-via(z, y)` | 0.872 | 78 |
