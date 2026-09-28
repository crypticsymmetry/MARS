# E33: MARS as an agent's episodic memory (incident response), noise 3

Config: {"seed": 1, "noise": 3, "templates": 40, "novel": 10, "incidents": 600, "seed_episodes": 60, "k": 5, "feedback_top": 3, "hidden_root_rate": 0.5}

| memory | fix@1 (learned) | fix@1 (raw) | fix@3 | root observed | root hidden | novel mechanism, first occurrence | top analogue = same mechanism | fix@1 by stream quarter |
|---|---|---|---|---|---|---|---|---|
| MARS (structure) | 0.646 | 0.585 | 0.731 | 0.914 | 0.422 | 0.00 (10) | 0.536 | 0.65 / 0.63 / 0.65 / 0.65 |
| MARS + names (identity 0.3) | 0.553 | 0.485 | 0.681 | 0.817 | 0.332 | 0.00 (10) | 0.412 | 0.57 / 0.54 / 0.54 / 0.55 |
| recall by names (identity only) | 0.231 | 0.134 | 0.325 | 0.317 | 0.158 | 0.10 (10) | 0.068 | 0.24 / 0.28 / 0.23 / 0.17 |
| popularity baseline | 0.068 | 0.068 | 0.068 | 0.067 | 0.068 | 0.00 (10) | — | 0.07 / 0.03 / 0.10 / 0.07 |

**Abstention by significance** (accept the top suggestion when z ≥ T):

| memory | T | coverage (known mechanisms) | precision of accepted | first occurrences of novel mechanisms abstained |
|---|---|---|---|---|
| MARS (structure) | 3.0 | 1.000 | 0.646 | 0.00 |
| MARS (structure) | 5.0 | 0.958 | 0.653 | 0.10 |
| MARS (structure) | 9.0 | 0.405 | 0.711 | 0.80 |
| MARS + names (identity 0.3) | 3.0 | 0.992 | 0.554 | 0.00 |
| MARS + names (identity 0.3) | 5.0 | 0.575 | 0.611 | 0.30 |
| MARS + names (identity 0.3) | 9.0 | 0.049 | 0.655 | 1.00 |
| recall by names (identity only) | 3.0 | 0.314 | 0.249 | 0.50 |
| recall by names (identity only) | 5.0 | 0.007 | 0.750 | 1.00 |
| recall by names (identity only) | 9.0 | 0.000 | nan | 1.00 |

**Rules the memory induced** (MARS, ≥ 10 feedback outcomes, precision ≥ 0.5):

| rule | precision | outcomes |
|---|---|---|
| `remedy-failover-db(x, y) ⇐ alerted-on(x, z) ∧ uses-db(z, y)` | 0.882 | 17 |
| `remedy-reboot-host(x, y) ⇐ alerted-on(x, z) ∧ runs-on(z, y)` | 0.867 | 15 |
| `remedy-renew-cert(x, y) ⇐ alerted-on(x, z) ∧ serves-cert(z, y)` | 0.833 | 12 |
| `remedy-kill-blocking-query(x, y) ⇐ alerted-on(x, z) ∧ uses-db(z, y)` | 0.750 | 12 |
| `remedy-fix-dns(x, y) ⇐ alerted-on(x, z) ∧ routes-via(z, y)` | 0.716 | 67 |
| `remedy-reroute-traffic(x, y) ⇐ alerted-on(x, z) ∧ routes-via(z, y)` | 0.706 | 34 |
| `remedy-replace-disk(x, y) ⇐ alerted-on(x, z) ∧ writes-to(z, y)` | 0.689 | 45 |
| `remedy-resync-clock(x, y) ⇐ alerted-on(x, z) ∧ runs-on(z, y)` | 0.650 | 40 |
| `remedy-dead-letter(x, y) ⇐ alerted-on(x, z) ∧ consumes(z, y)` | 0.634 | 41 |
| `remedy-free-disk(x, y) ⇐ alerted-on(x, z) ∧ writes-to(z, y)` | 0.574 | 47 |
| `remedy-reissue-cert: unlinked` | 0.500 | 50 |
