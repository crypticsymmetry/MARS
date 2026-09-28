#!/usr/bin/env python3
"""E33: MARS as the episodic memory of an incident-response agent.

    python3 tools/e33_agent_memory.py OUT_DIR [--incidents 600] [--templates 40] [--novel 10] [--noise 0] [--seed 1]
    python3 tools/e33_agent_memory.py --demo      # narrated walk-through of a few incidents
    python3 tools/e33_agent_memory.py OUT_DIR --seeds 1,2,3,4,5   # all noise levels × seeds (mean ± sd; per-incident outcomes)
    python3 tools/e33_agent_memory.py OUT_DIR --llm 150 --noise 2   # + LLM agent baselines (GLM-5.3-Flash via OpenRouter)

An agent resolves a stream of incidents (outages) with MARS (Python bindings)
as its memory:

* Incidents come from failure *mechanisms*: a root cause on a resource (disk,
  database, certificate, queue, cache, network, host) propagating through a
  chain of service symptoms, e.g. full(disk) -> write-error(svc) -> crash(svc).
  The fix is determined by the root cause and applied to the root entity:
  (remedy-free-disk INC DISK). Service/host names recur across *different*
  mechanisms, so surface similarity misleads.
* In half the incidents the root cause is not observed (its event and causal
  link are missing); the agent must still infer the fix and its target.
* The agent asks the memory for suggestions, applies the top remedy, gets the
  outcome, gives feedback on the top-3 remedy suggestions, and stores the
  resolved episode. Memory starts with 2 episodes of each non-novel mechanism;
  `--novel` mechanisms appear only in the stream (their first occurrence has
  no precedent: the agent should abstain, judged by the significance z).

Memories compared (same stream): MARS (structural fingerprints + mapping),
recall by names (identity channel only: lexical memory + the same mapper), the
fused mix, and a popularity baseline (most frequent remedy kind applied to the
first entity of the right type). Reports fix accuracy (kind and target) over the
stream, with and without the observed root cause, learned vs raw ranking,
abstention by z, and the rules the memory induced.
"""

import json
import os
import random
import sys
from collections import Counter, defaultdict

import mars

# Resources: type -> (context relation linking a service to it, name pool, root-cause events with fixes).
RESOURCES = {
    "disk": ("writes-to", [f"disk-{i}" for i in range(12)], {"disk-full": "free-disk", "slow-io": "replace-disk"}),
    "db": ("uses-db", [f"db-{n}" for n in ("orders", "users", "billing", "catalog", "audit", "search")], {"pool-exhausted": "raise-pool", "deadlock": "kill-blocking-query", "replica-lag": "failover-db"}),
    "cert": ("serves-cert", [f"cert-{n}" for n in ("www", "api", "internal", "partner", "admin")], {"cert-expired": "renew-cert", "cert-revoked": "reissue-cert"}),
    "queue": ("consumes", [f"queue-{n}" for n in ("events", "email", "jobs", "billing", "audit")], {"backlog": "add-consumers", "poison-message": "dead-letter"}),
    "cache": ("reads-cache", [f"cache-{n}" for n in ("session", "product", "price", "feed")], {"cache-stampede": "warm-cache", "eviction-storm": "resize-cache"}),
    "net": ("routes-via", [f"net-{n}" for n in ("east", "west", "edge")], {"packet-loss": "reroute-traffic", "dns-failure": "fix-dns"}),
    "host": ("runs-on", [f"host-{i}" for i in range(10)], {"cpu-saturated": "scale-out", "clock-skew": "resync-clock", "kernel-panic": "reboot-host"}),
}
SERVICES = [f"svc-{n}" for n in ("payments", "checkout", "search", "auth", "cart", "profile", "orders", "catalog", "email", "gateway", "feed", "billing", "reports", "inventory", "shipping", "reviews")]
SYMPTOMS = ["write-error", "crash", "oom", "restart-loop", "high-latency", "error-5xx", "timeout", "tls-failure", "auth-failure", "stale-reads", "retry-storm", "connection-refused"]
CONTEXT = [r[0] for r in RESOURCES.values()] + ["depends-on"]


def declarations():
    lines = ["(defpredicate cause :arity 2 :kind relation)", "(defpredicate alerted-on :arity 2 :kind relation)"]
    lines += [f"(defpredicate {c} :arity 2 :kind relation)" for c in CONTEXT]
    events = {e for r in RESOURCES.values() for e in r[2]} | set(SYMPTOMS)
    lines += [f"(defpredicate {e} :arity 1 :kind relation)" for e in sorted(events)]
    fixes = {f for r in RESOURCES.values() for f in r[2].values()}
    lines += [f"(defpredicate remedy-{f} :arity 2 :kind relation)" for f in sorted(fixes)]
    return "\n".join(lines) + "\n"


def make_templates(n, rng):
    """A mechanism: (resource type, root event, symptom chain, propagates to a dependent service?)."""
    seen, out = set(), []
    while len(out) < n:
        rtype = rng.choice(sorted(RESOURCES))
        root = rng.choice(sorted(RESOURCES[rtype][2]))
        chain = tuple(rng.sample(SYMPTOMS, rng.choice([2, 3])))
        dep = rng.random() < 0.4
        t = (rtype, root, chain, dep)
        if t not in seen:
            seen.add(t)
            out.append(t)
    return out


def instance(t, iid, rng):
    """Facts of one incident of mechanism t (with its resolution) and the pieces the agent may not see."""
    rtype, root, chain, dep = t
    rel, pool, fixes = RESOURCES[rtype]
    res = rng.choice(pool)
    s1, s2 = rng.sample(SERVICES, 2)
    inc = f"inc-{iid}"
    facts = [f"({rel} {s1} {res})"]
    events = [f"({root} {res})"]
    svc = s1
    for k, sym in enumerate(chain):
        if dep and k == len(chain) - 1:
            facts.append(f"(depends-on {s2} {s1})")
            svc = s2
        events.append(f"({sym} {svc})")
    causes = [f"(cause {a} {b})" for a, b in zip(events, events[1:])]
    facts += events + causes + [f"(alerted-on {inc} {svc})"]
    # Context noise: unrelated deployment facts about other services.
    for _ in range(2):
        orel, (rel2, pool2, _) = rng.choice(sorted(RESOURCES.items()))
        facts.append(f"({rel2} {rng.choice(SERVICES)} {rng.choice(pool2)})")
    remedy = f"(remedy-{fixes[root]} {inc} {res})"
    root_facts = [events[0], causes[0]]
    return inc, sorted(set(facts)), remedy, root_facts


def add_noise(facts, root_facts, inc, level, templates, rng):
    """Severity `level`: that many concurrent unrelated anomalies (a root event and
    its first symptom on other entities, with their context, no alert), and that
    many observed facts dropped at random (never the alert or the root facts,
    which are hidden separately)."""
    facts = list(facts)
    for _ in range(level):
        rtype, root, chain, _ = rng.choice(templates)
        rel, pool, _ = RESOURCES[rtype]
        res, svc = rng.choice(pool), rng.choice(SERVICES)
        a, b = f"({root} {res})", f"({chain[0]} {svc})"
        facts += [f"({rel} {svc} {res})", a, b, f"(cause {a} {b})"]
    droppable = [f for f in facts if not f.startswith("(alerted-on") and f not in root_facts]
    for f in rng.sample(droppable, min(level, len(droppable))):
        facts.remove(f)
    return sorted(set(facts))


def defcase(name, facts):
    return f"(defcase {name} " + " ".join(facts) + ")"


def remedies(suggestions, inc):
    """Remedy suggestions for this incident, in rank order: (text, dict)."""
    return [s for s in suggestions if s["text"].startswith("(remedy-") and s["text"].split()[1] == inc]


def run(templates, novel_ids, stream, seed_eps, identity, popularity=False):
    """One memory over the stream. Returns per-incident records."""
    src = declarations() + "\n".join(defcase(n, f) for n, f in seed_eps)
    e = mars.Engine(src, first_order=True, identity_weight=identity)
    fix_count = Counter(r.split()[0] for _, f in seed_eps for r in f if r.startswith("(remedy-"))
    seen_t = Counter(t for t in range(len(templates)) if t not in novel_ids for _ in range(2))
    recs = []
    for i, (ti, inc, facts, remedy, root_facts, hidden) in enumerate(stream):
        observed = [f for f in facts if not (hidden and f in root_facts)]
        q = f"case-{inc}"
        e.add_case(defcase(q, observed))
        e.remove_case(q)  # a query, not a memory
        hits, z = e.query(q, k=5)
        sug = remedies(e.suggest(q, k=5), inc)
        if popularity:
            # Most frequent remedy kind so far, applied to the first entity of its resource type.
            kind = fix_count.most_common(1)[0][0][len("(remedy-"):]
            rel = next(r[0] for r in RESOURCES.values() if kind in r[2].values())
            target = next((f.split()[2].rstrip(")") for f in observed if f.startswith(f"({rel} ")), None)
            learned = raw = [f"(remedy-{kind} {inc} {target})"] if target else []
        else:
            learned = [s["text"] for s in sug]
            raw = [s["text"] for s in sorted(sug, key=lambda s: (-s["weight"], s["text"]))]
        rec = {"i": i, "template": ti, "novel_first": ti in novel_ids and seen_t[ti] == 0, "hidden_root": hidden, "z": z,
               "learned_top1": bool(learned) and learned[0] == remedy, "learned_top3": remedy in learned[:3],
               "raw_top1": bool(raw) and raw[0] == remedy, "answered": bool(learned),
               "top_analogue_same_mechanism": bool(hits) and hits[0][0].startswith("ep-") and int(hits[0][0].split("-t")[1]) == ti if hits else False}
        recs.append(rec)
        # Feedback on the top-3 remedy suggestions, then store the resolved episode.
        if not popularity:
            for s in sug[:3]:
                e.feedback(q, s["text"], s["text"] == remedy)
        e.add_case(defcase(f"ep-{inc}-t{ti}", facts + [remedy]))
        fix_count[remedy.split()[0]] += 1
        seen_t[ti] += 1
    rules = [] if popularity else e.induced_rules(min_n=10, min_precision=0.5)
    return recs, rules


def build(args_seed, n_templates, n_novel, n_incidents, noise=0):
    rng = random.Random(args_seed)
    templates = make_templates(n_templates, rng)
    novel_ids = set(rng.sample(range(n_templates), n_novel))
    seed_eps = []
    k = 0
    for ti, t in enumerate(templates):
        if ti in novel_ids:
            continue
        for _ in range(2):
            inc, facts, remedy, _ = instance(t, f"s{k}", rng)
            seed_eps.append((f"ep-{inc}-t{ti}", facts + [remedy]))
            k += 1
    stream = []
    for i in range(n_incidents):
        ti = rng.randrange(n_templates)
        inc, facts, remedy, root_facts = instance(templates[ti], str(i), rng)
        observed_noise = add_noise(facts, root_facts, inc, noise, templates, rng) if noise else facts
        stream.append((ti, inc, observed_noise, remedy, root_facts, rng.random() < 0.5))
    # The query case gets its own incident id: rewrite inc-<i> -> q-inc-<i> in its facts and remedy.
    stream = [(ti, f"q-{inc}", [f.replace(inc, f"q-{inc}") for f in facts], remedy.replace(inc, f"q-{inc}"), [f.replace(inc, f"q-{inc}") for f in rf], h) for ti, inc, facts, remedy, rf, h in stream]
    return templates, novel_ids, seed_eps, stream


def summarize(recs):
    def acc(sel, key):
        sel = list(sel)
        return sum(r[key] for r in sel) / len(sel) if sel else float("nan")
    known = [r for r in recs if not r["novel_first"]]
    out = {
        "fix_top1_learned": acc(known, "learned_top1"),
        "fix_top1_raw": acc(known, "raw_top1"),
        "fix_top3_learned": acc(known, "learned_top3"),
        "root_observed_top1": acc((r for r in known if not r["hidden_root"]), "learned_top1"),
        "root_hidden_top1": acc((r for r in known if r["hidden_root"]), "learned_top1"),
        "top_analogue_same_mechanism": acc(known, "top_analogue_same_mechanism"),
        "n_known": len(known),
        "n_novel_first": sum(r["novel_first"] for r in recs),
        "novel_first_top1": acc((r for r in recs if r["novel_first"]), "learned_top1"),
    }
    q = len(known) // 4
    out["curve_top1"] = [acc(known[j * q:(j + 1) * q], "learned_top1") for j in range(4)]
    # Abstention by significance z: accept when z >= T.
    ab = {}
    for T in (3.0, 5.0, 9.0):
        acc_known = [r for r in known if r["z"] is not None and r["z"] >= T]
        nov = [r for r in recs if r["novel_first"]]
        ab[str(T)] = {"coverage_known": len(acc_known) / max(1, len(known)), "precision_accepted": acc(acc_known, "learned_top1"),
                      "novel_abstained": sum(1 for r in nov if r["z"] is None or r["z"] < T) / max(1, len(nov))}
    out["abstention"] = ab
    return out


def main():
    if "--demo" in sys.argv:
        return demo()
    if "--seeds" in sys.argv:
        return seeds_eval(sys.argv[1], [int(x) for x in sys.argv[sys.argv.index("--seeds") + 1].split(",")])
    if "--llm" in sys.argv:
        a = lambda k, d: type(d)(sys.argv[sys.argv.index(k) + 1]) if k in sys.argv else d
        return llm_eval(sys.argv[1], a("--noise", 2), a("--llm", 150), a("--seed", 1))
    out_dir = sys.argv[1]
    arg = lambda k, d: type(d)(sys.argv[sys.argv.index(k) + 1]) if k in sys.argv else d
    seed, n_t, n_nov, n_inc, noise = arg("--seed", 1), arg("--templates", 40), arg("--novel", 10), arg("--incidents", 600), arg("--noise", 0)
    templates, novel_ids, seed_eps, stream = build(seed, n_t, n_nov, n_inc, noise)
    configs = [("MARS (structure)", 0.0, False), ("MARS + names (identity 0.3)", 0.3, False), ("recall by names (identity only)", 1.0, False), ("popularity baseline", 0.0, True)]
    results = {}
    rules_out = []
    for name, lam, pop in configs:
        recs, rules = run(templates, novel_ids, stream, seed_eps, lam, pop)
        results[name] = summarize(recs)
        if name == "MARS (structure)":
            rules_out = rules
        print(name, json.dumps({k: v for k, v in results[name].items() if k != "abstention"}), file=sys.stderr)
    os.makedirs(out_dir, exist_ok=True)
    cfg = {"seed": seed, "noise": noise, "templates": n_t, "novel": n_nov, "incidents": n_inc, "seed_episodes": len(seed_eps), "k": 5, "feedback_top": 3, "hidden_root_rate": 0.5}
    json.dump({"config": cfg, "results": results, "induced_rules": rules_out}, open(f"{out_dir}/E33-noise{noise}.json", "w"), indent=1)
    L = [f"# E33: MARS as an agent's episodic memory (incident response), noise {noise}\n", f"Config: {json.dumps(cfg)}\n",
         "| memory | fix@1 (learned) | fix@1 (raw) | fix@3 | root observed | root hidden | novel mechanism, first occurrence | top analogue = same mechanism | fix@1 by stream quarter |", "|---|---|---|---|---|---|---|---|---|"]
    for name, r in results.items():
        same = "—" if "popularity" in name else f"{r['top_analogue_same_mechanism']:.3f}"
        L.append(f"| {name} | {r['fix_top1_learned']:.3f} | {r['fix_top1_raw']:.3f} | {r['fix_top3_learned']:.3f} | {r['root_observed_top1']:.3f} | {r['root_hidden_top1']:.3f} | {r['novel_first_top1']:.2f} ({r['n_novel_first']}) | {same} | {' / '.join(f'{x:.2f}' for x in r['curve_top1'])} |")
    L.append("\n**Abstention by significance** (accept the top suggestion when z ≥ T):\n\n| memory | T | coverage (known mechanisms) | precision of accepted | first occurrences of novel mechanisms abstained |\n|---|---|---|---|---|")
    for name, r in results.items():
        if "popularity" in name:
            continue
        for T, a in r["abstention"].items():
            L.append(f"| {name} | {T} | {a['coverage_known']:.3f} | {a['precision_accepted']:.3f} | {a['novel_abstained']:.2f} |")
    L.append("\n**Rules the memory induced** (MARS, ≥ 10 feedback outcomes, precision ≥ 0.5):\n\n| rule | precision | outcomes |\n|---|---|---|")
    for rule, p, n in rules_out[:20]:
        L.append(f"| `{rule}` | {p:.3f} | {n:.0f} |")
    open(f"{out_dir}/E33-noise{noise}.md", "w").write("\n".join(L) + "\n")
    print("\n".join(L))


LLM_PROMPT = """You are an incident-response agent. Below are past resolved incidents from your memory, as facts (s-expressions); each ends with the remedy that fixed it, (remedy-KIND INCIDENT TARGET). Then a new incident. Its root cause may be unobserved, and it may contain unrelated anomalies.

{memory}

New incident {inc}:
{incident}

Decide the remedy for {inc}. Reply with exactly one line: (remedy-KIND {inc} TARGET), where TARGET is an entity of the new incident."""


def llm_call(prompt, model="z-ai/glm-5.3-flash", timeout=120, retries=3):
    """One chat completion (OpenRouter; key from $OPENROUTER_API_KEY, never stored). Thread-safe."""
    import time
    import urllib.request
    key = os.environ["OPENROUTER_API_KEY"]
    body = json.dumps({"model": model, "messages": [{"role": "user", "content": prompt}], "temperature": 0, "reasoning": {"effort": "low"}}).encode()
    for i in range(retries):
        try:
            req = urllib.request.Request("https://openrouter.ai/api/v1/chat/completions", data=body, headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
            with urllib.request.urlopen(req, timeout=timeout) as r:
                d = json.load(r)
            msg = (d.get("choices") or [{}])[0].get("message", {}).get("content") or ""
            if msg.strip():
                return msg, d.get("usage", {})
        except Exception as e:
            print(f"  LLM attempt {i + 1}: {str(e)[:120]}", file=sys.stderr, flush=True)
        time.sleep(3 * (i + 1))
    return "", {}


def llm_eval(out_dir, noise, n_eval, seed, n_t=40, n_nov=10, n_inc=600):
    """LLM agent (GLM-5.3-Flash) given the top-5 episodes retrieved by names or by MARS,
    on every (n_inc // n_eval)-th incident of the stream; MARS's own answer on the same subset."""
    from concurrent.futures import ThreadPoolExecutor
    import re
    templates, novel_ids, seed_eps, stream = build(seed, n_t, n_nov, n_inc, noise)
    src = declarations() + "\n".join(defcase(n, f) for n, f in seed_eps)
    engines = {"MARS": mars.Engine(src, first_order=True), "names": mars.Engine(src, first_order=True, identity_weight=1.0)}
    episodes = dict(seed_eps)
    step = max(1, n_inc // n_eval)
    jobs, mars_ans = [], []
    for i, (ti, inc, facts, remedy, root_facts, hidden) in enumerate(stream):
        observed = [f for f in facts if not (hidden and f in root_facts)]
        q = f"case-{inc}"
        for name, e in engines.items():
            e.add_case(defcase(q, observed))
            e.remove_case(q)
            hits, _ = e.query(q, k=5)
            if name == "MARS":
                sug = remedies(e.suggest(q, k=5), inc)
                for s_ in sug[:3]:
                    e.feedback(q, s_["text"], s_["text"] == remedy)
                if i % step == 0:
                    mars_ans.append(bool(sug) and sug[0]["text"] == remedy)
            if i % step == 0:
                mem = "\n\n".join(f"Past incident {h}:\n" + "\n".join(episodes[h]) for h, _ in hits)
                jobs.append((i, name, remedy, LLM_PROMPT.format(memory=mem, inc=inc, incident="\n".join(observed))))
        ep = f"ep-{inc}-t{ti}"
        episodes[ep] = facts + [remedy]
        for e in engines.values():
            e.add_case(defcase(ep, facts + [remedy]))
    print(f"[e33-llm] {len(jobs)} LLM calls", file=sys.stderr, flush=True)
    with ThreadPoolExecutor(8) as ex:
        outs = list(ex.map(lambda j: llm_call(j[3]), jobs))
    res = defaultdict(list)
    tokens, cost, failed = 0, 0.0, 0
    for (i, name, remedy, _), (msg, usage) in zip(jobs, outs):
        cost += usage.get("cost", 0.0)
        failed += not msg
        m = re.findall(r"\(remedy-[^()]*\)", msg)
        ans = m[-1].strip() if m else ""
        res[name].append(ans == remedy)
        tokens += usage.get("prompt_tokens", 0) + usage.get("completion_tokens", 0)
    r = {"n": len(mars_ans), "MARS alone": sum(mars_ans) / len(mars_ans), "LLM + names-retrieved episodes": sum(res["names"]) / len(res["names"]), "LLM + MARS-retrieved episodes": sum(res["MARS"]) / len(res["MARS"]), "tokens": tokens, "cost_usd": round(cost, 4), "failed_calls": failed, "model": "z-ai/glm-5.3-flash", "reasoning": "low"}
    os.makedirs(out_dir, exist_ok=True)
    json.dump({"config": {"seed": seed, "noise": noise, "n_eval": n_eval, "incidents": n_inc, "templates": n_t, "novel": n_nov}, "results": r}, open(f"{out_dir}/E33-llm-noise{noise}.json", "w"), indent=1)
    print(json.dumps(r, indent=1))


def seeds_eval(out_dir, seed_list, noises=(0, 1, 2, 3)):
    """Each seed = an independent set of mechanisms, stream and noise. Reports fix@1 on known
    mechanisms (mean ± sd over seeds) and keeps seed-1 per-incident outcomes for paired tests."""
    sys.path.insert(0, os.path.dirname(__file__))
    from stats import seeds as msd
    configs = [("MARS (structure)", 0.0, False), ("MARS + names (identity 0.3)", 0.3, False), ("recall by names (identity only)", 1.0, False), ("popularity baseline", 0.0, True)]
    across, paired = {}, {}
    for noise in noises:
        vals = defaultdict(list)
        for sd in seed_list:
            templates, novel_ids, seed_eps, stream = build(sd, 40, 10, 600, noise)
            for name, lam, pop in configs:
                recs, _ = run(templates, novel_ids, stream, seed_eps, lam, pop)
                known = [r for r in recs if not r["novel_first"]]
                vals[name].append(sum(r["learned_top1"] for r in known) / len(known))
                if sd == seed_list[0]:
                    paired.setdefault(str(noise), {})[name] = [float(r["learned_top1"]) for r in known]
            print(f"[e33] noise {noise} seed {sd} done", file=sys.stderr, flush=True)
        across[str(noise)] = {name: msd(v) for name, v in vals.items()}
    os.makedirs(out_dir, exist_ok=True)
    json.dump({"seeds": seed_list, "memories": [c[0] for c in configs], "across_seeds": across, "paired": paired, "config": {"templates": 40, "novel": 10, "incidents": 600, "hidden_root_rate": 0.5}}, open(f"{out_dir}/E33-seeds.json", "w"), indent=1)
    for noise, v in across.items():
        print(noise, {k: f"{m:.3f}±{s:.3f}" for k, (m, s) in v.items()})


def demo():
    """A narrated walk-through: memory, suggestion with provenance, feedback, standing query."""
    rng = random.Random(7)
    t_disk = ("disk", "disk-full", ("write-error", "crash"), False)
    t_cert = ("cert", "cert-expired", ("tls-failure", "error-5xx"), True)
    eps = []
    for k, t in enumerate([t_disk, t_disk, t_cert, t_cert]):
        inc, facts, remedy, _ = instance(t, f"past{k}", rng)
        eps.append((f"ep-{inc}", facts + [remedy]))
    e = mars.Engine(declarations() + "\n".join(defcase(n, f) for n, f in eps), first_order=True)
    print("Memory:", e.cases())
    inc, facts, remedy, root = instance(t_disk, "now", rng)
    observed = [f for f in facts if f not in root]  # root cause not observed
    print("\nNew incident (root cause unobserved):\n  " + "\n  ".join(observed))
    e.add_case(defcase("case-now", [f.replace(inc, "q-now") for f in observed]))
    hits, z = e.query("case-now", k=3)
    print(f"\nClosest past incidents: {[(h, round(s, 3)) for h, s in hits]}  (significance z = {z})")
    for s in remedies(e.suggest("case-now", k=3), "q-now")[:3]:
        print(f"  suggest {s['text']}  support {s['support']}  reliability {s['reliability']:.2f}  via {s['analogues']}")
    sq = e.watch("case-now", k=3)
    print("\nStanding query registered; a new similar incident is resolved and stored...")
    inc2, facts2, remedy2, _ = instance(t_disk, "later", rng)
    e.add_case(defcase(f"ep-{inc2}", facts2 + [remedy2]))
    print("  now the closest are", [n for n, _ in e.top(sq)])
    for text, sup in e.infer(sq)[:4]:
        print(f"  corroborated: {text} (support {sup})")


if __name__ == "__main__":
    main()
