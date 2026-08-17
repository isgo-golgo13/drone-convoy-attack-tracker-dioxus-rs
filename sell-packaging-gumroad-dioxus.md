# Selling the Dioxus Edition on Gumroad — packaging & launch

The **second product**: the same drone convoy tracking system, Dioxus 0.7
frontend, backend byte-identical to the Leptos edition. This document covers
only what is *different* from the Leptos runbook (`sell-packaging-gumroad.md`
in the Leptos repo) — reuse that for the mechanics (private repo, custom
LICENSE, `git archive` from a tag, PDF stamping, license keys, update flow).

---

## 0 — Positioning (get this right; it decides whether the second sale exists)

Do NOT sell it as "the same thing again." Sell it as **the framework decision,
made real**: one production system, both major Rust web frameworks, so the
buyer evaluating Leptos-vs-Dioxus doesn't compare two TODO apps — they compare
two builds of a real distributed system with a real map, real animation, a
real leaderboard, and a real ops backend behind both.

The one-line pitch: *"Same system. Same backend, byte for byte. Same pixels.
Different framework. Now decide."*

Three buyer profiles, and the product speaks to each:
1. **Owns the Leptos edition** — wants the second framework to compare, or is
   moving a team to Dioxus. Bundle price (below).
2. **Dioxus-first buyer** — never wanted Leptos; this is their edition of the
   $150 product. Standalone price.
3. **Deciding** — buys the bundle *because* the comparison is the value.

## 1 — Pricing

| SKU | Price | Notes |
|---|---|---|
| Dioxus edition, single seat | **$150** | same as Leptos: same system, same depth |
| Dioxus edition, team (5) | **$375** | |
| **Bundle: Leptos + Dioxus, single** | **$225** | $75 off; the comparison IS the value |
| Bundle, team (5) | **$560** | |
| Upgrade code for Leptos owners | **$75** | Gumroad "offer code" restricted to prior buyers' emails |

Launch code `LAUNCH` $25 off for 14 days, same as before. Never discount below
the bundle math (single-edition price must always exceed bundle − other
edition, or the bundle stops making sense).

## 2 — What ships (the zip)

Built the same way as the Leptos edition — `git archive` from a tag of the
private repo — containing:

    drone-convoy-attack-tracker-dioxus-rs/     source (backend identical, Dioxus frontend)
    LICENSE  VERSION  CHANGELOG.md  README.md
    docs/drone-convoy-1..5.png                 same 5 screenshots (they ARE identical — say so)
    + the tutorial PDF as a second Gumroad file

Buyer's-eye test before every upload: unzip to a clean dir, `make setup &&
make build && make serve`, dashboard on :3000, pick a theater, drones move.

## 3 — The tutorial PDF (Dioxus edition): reuse ~65%, write ~35%

The Leptos tutorial's chapters 1 (it flies), 2 (one type system across the
wire), 3 (ScyllaDB), 4 (Redis), 5 (GraphQL API), 6 (Rust the way this codebase
does it), 8 (simulator/convoy service), 9 (KinD/Cilium deploy), 10 (appendices)
are **framework-agnostic and reused verbatim** — that's the payoff of the
architecture. Rewrite/add:

- **Ch 7 — Dioxus frontend** (replaces the Leptos ch 7): `rsx!`, `Signal` and
  the `Copy` AppState pattern, `use_context_provider`, `use_effect` vs
  `use_future` vs `spawn`, `for`+key rendering with the composite-key lesson,
  `asset!()`, `dx serve` hot reload. Same components, same screenshots.
- **NEW Ch 7b — The port, construct for construct.** The translation table
  from the README, expanded with a code sidebar per row: the Leptos snippet
  on the left, the Dioxus snippet on the right, from *this* codebase (the
  header clock, the leaderboard row, the map mount, the tasking effect).
- **NEW Ch 7c — Three rules Dioxus taught us (war stories).** Write-during-
  render, guard-across-await, raw-callback writes — each with the actual
  symptom (OFFLINE forever / RefCell already borrowed / airframes stacked at
  the IP), the panic message, the mechanism, and the fix. This is the chapter
  no one else can write, and it's the honest answer to "which is harder."
- **NEW Ch 11 — Leptos vs Dioxus, having built both.** The comparison the
  buyer paid for. Honest, concrete, both directions (see §5 below for the
  author's position). End with "which should you pick" as a decision table
  by team/product shape, not a verdict.

Target: same ~85 pages; ~30 pages new/rewritten.

## 4 — Cross-promotion (both products lift each other)

- Each product's Gumroad page links the other and the bundle.
- The Leptos edition's README gets one line: "A Dioxus edition of this system
  exists — same backend, same pixels; see …" (and vice versa — already in the
  Dioxus README intro).
- One shared 30-second recording: split-screen, Leptos left / Dioxus right,
  same sortie, retask both to Iraq at once. That single clip is the entire
  argument. Cover media on BOTH pages.
- Announce the Dioxus edition where the Dioxus community lives (their
  Discord #showcase, r/rust) — lead with the "three rules" write-up as a
  technical post; the product link is second.

## 5 — Positioning honesty: the author's actual view (use it in ch 11)

Having built the same system twice, the fair summary is: **Leptos is the
richer, more precise framework; Dioxus is the more approachable and more
portable one.** Leptos's fine-grained signals are lock-free and let you do
things (write from a raw callback, hold a read across an await) that Dioxus's
scheduler-owned signals forbid — which is exactly why the port hit three
runtime panics that the Leptos build never had. In exchange Dioxus gives a
`rsx!` syntax most React refugees read instantly, a genuinely better dev loop
(`dx` hot-patching), one codebase to desktop/mobile, and a runtime that
*tells you* when you've done something unsafe instead of silently letting you.
Say both halves. Buyers of a comparison product can smell a thumb on the
scale, and this repo's credibility is that both builds run.

## 6 — Launch checklist (delta from the Leptos one)

- [ ] Both repos private, both tagged (`v1.0.0` each), both zips `git archive`d
- [ ] Bundle product created on Gumroad with both zips + both PDFs
- [ ] Upgrade offer code created, restricted to Leptos buyers
- [ ] Split-screen recording uploaded as cover media on all three pages
- [ ] Ch 7/7b/7c/11 written; rest of PDF reused; version 1.0.0 on cover
- [ ] Buyer's-eye test passed from the clean unzip of the Dioxus zip
- [ ] Cross-links in both READMEs and both Gumroad descriptions
- [ ] Announce: Dioxus Discord, r/rust, X/LinkedIn (recording), TWiR
