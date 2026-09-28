# E33: MARS as an agent's episodic memory (incident response), noise 2

Config: {"seed": 1, "noise": 2, "templates": 40, "novel": 10, "incidents": 600, "seed_episodes": 60, "k": 5, "feedback_top": 3, "hidden_root_rate": 0.5}

| memory | fix@1 (learned) | fix@1 (raw) | fix@3 | root observed | root hidden | novel mechanism, first occurrence | top analogue = same mechanism | fix@1 by stream quarter |
|---|---|---|---|---|---|---|---|---|
| MARS (structure) | 0.761 | 0.732 | 0.836 | 0.954 | 0.587 | 0.30 (10) | 0.646 | 0.74 / 0.76 / 0.74 / 0.80 |
| MARS + names (identity 0.3) | 0.722 | 0.666 | 0.792 | 0.921 | 0.542 | 0.20 (10) | 0.536 | 0.71 / 0.71 / 0.71 / 0.76 |
| recall by names (identity only) | 0.315 | 0.210 | 0.400 | 0.432 | 0.210 | 0.10 (10) | 0.095 | 0.30 / 0.31 / 0.33 / 0.33 |
| popularity baseline | 0.078 | 0.078 | 0.078 | 0.086 | 0.071 | 0.00 (10) | — | 0.05 / 0.10 / 0.08 / 0.08 |

**Abstention by significance** (accept the top suggestion when z ≥ T):

| memory | T | coverage (known mechanisms) | precision of accepted | first occurrences of novel mechanisms abstained |
|---|---|---|---|---|
| MARS (structure) | 3.0 | 1.000 | 0.761 | 0.00 |
| MARS (structure) | 5.0 | 0.956 | 0.762 | 0.00 |
| MARS (structure) | 9.0 | 0.458 | 0.822 | 0.60 |
| MARS + names (identity 0.3) | 3.0 | 0.995 | 0.722 | 0.00 |
| MARS + names (identity 0.3) | 5.0 | 0.719 | 0.748 | 0.30 |
| MARS + names (identity 0.3) | 9.0 | 0.075 | 0.886 | 1.00 |
| recall by names (identity only) | 3.0 | 0.317 | 0.348 | 0.60 |
| recall by names (identity only) | 5.0 | 0.010 | 0.333 | 1.00 |
| recall by names (identity only) | 9.0 | 0.000 | nan | 1.00 |

**Rules the memory induced** (MARS, ≥ 10 feedback outcomes, precision ≥ 0.5):

| rule | precision | outcomes |
|---|---|---|
| `remedy-resync-clock(x, y) ⇐ alerted-on(x, z) ∧ runs-on(z, y)` | 0.897 | 39 |
| `remedy-warm-cache(x, y) ⇐ alerted-on(x, z) ∧ reads-cache(z, y)` | 0.882 | 17 |
| `remedy-dead-letter(x, y) ⇐ alerted-on(x, z) ∧ consumes(z, y)` | 0.873 | 55 |
| `remedy-renew-cert(x, y) ⇐ alerted-on(x, z) ∧ serves-cert(z, y)` | 0.867 | 15 |
| `remedy-replace-disk(x, y) ⇐ alerted-on(x, z) ∧ writes-to(z, y)` | 0.829 | 41 |
| `remedy-resize-cache(x, y) ⇐ alerted-on(x, z) ∧ reads-cache(z, y)` | 0.818 | 11 |
| `remedy-failover-db(x, y) ⇐ alerted-on(x, z) ∧ uses-db(z, y)` | 0.800 | 15 |
| `remedy-reroute-traffic(x, y) ⇐ alerted-on(x, z) ∧ routes-via(z, y)` | 0.792 | 48 |
| `remedy-reboot-host(x, y) ⇐ alerted-on(x, z) ∧ runs-on(z, y)` | 0.750 | 24 |
| `remedy-free-disk(x, y) ⇐ alerted-on(x, z) ∧ writes-to(z, y)` | 0.690 | 29 |
| `remedy-failover-db: unlinked` | 0.680 | 25 |
| `remedy-fix-dns(x, y) ⇐ alerted-on(x, z) ∧ routes-via(z, y)` | 0.658 | 73 |
| `remedy-add-consumers(x, y) ⇐ alerted-on(x, z) ∧ consumes(z, y)` | 0.650 | 20 |
| `remedy-reissue-cert: unlinked` | 0.565 | 46 |
| `remedy-resync-clock: unlinked` | 0.531 | 32 |
| `remedy-scale-out: unlinked` | 0.524 | 21 |
