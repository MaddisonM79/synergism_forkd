<!--
Generated 2026-06-06 by a 4-agent backend-API audit of the legacy TS client + MSW mocks,
cross-referenced against the target platform (~/github/lmam_api, api.lmam.tech).
Purpose: catalog every backend call the game makes today, and plan what we build —
with the explicit goal of leaving the contract open to change. Analysis only; no code written.
This file is planning scratch — keep, move into lmam_api, or delete freely.
-->

# Synergism Forkd — Backend API: Current Contract + Build Plan

## 0. TL;DR

**There is no server source anywhere in this repo.** `legacy/` contains only the *client* and its MSW (Mock Service Worker) *mock handlers*. The mocks were written to develop the UI offline, and they double as the single best record of the request/response contract. So everything the game expects from a backend has been reconstructed from (a) the client call sites and (b) the mocks. Confidence is high where a mock exists, marked **INFERRED** where only the client's parsing pins the shape.

**The backend the client points at by default is `https://synergism.cc` / `wss://synergism.cc`** — the original project's production server, owned by the upstream author. Both `apiBaseUrl` and `wsBaseUrl` are overridable at build time (`VITE_API_BASE_URL`, `VITE_WS_BASE_URL`, `VITE_CANONICAL_HOST`). The *paths*, however, are hardcoded in the client.

**The surface is ~30 HTTP endpoints across 5 domains + 1 WebSocket** (the "consumables" channel, ~19 message types). Domains: **Identity/Auth**, **Cloud Saves**, **Live game services** (events, quark-bonus, contributors, i18n), **Messages/news**, and a large **PseudoCoins monetization stack** (Stripe / PayPal / NOWPayments / Fourthwall merch + the consumables WS).

**The central reframing (this is the important part):** in a normal port you must match the existing API because the client is fixed. **Here both ends are being rebuilt** — the Rust/Dioxus client is greenfield, and the server will be proprietary (built into `api.lmam.tech`). So the TS contract below is a **reference design, not a binding constraint.** It tells us *what services the game needs*; it does **not** dictate the wire format. We are free to redesign paths, auth, and payloads wholesale. The only thing that would force byte-compatibility is a decision to run the *old TS client* against the new server during transition — an open question (§10, Q1).

**Target platform fit is excellent.** `api.lmam.tech` (`lmam_api`) is a Cloudflare Worker + Hono + Prisma/Neon + Hanko monorepo that *already* has the two hard primitives this needs: a **Durable Object** (for the consumables WebSocket) and an **S3 client + presigner** (for cloud-save blobs). Synergism becomes a new app (`synergism-db` + `synergism-service` + `routes/synergism.ts`, env prefix `SYN_`).

### The six decisions this plan needs from you (details in §10)
1. **Compatibility target** — must the legacy TS client keep working against the new API (byte-compatible paths/shapes), or is the Rust client the only consumer (free redesign)?
2. **Auth model** — adopt `lmam_api`'s Hanko (JWT/Principal) and drop the cookie/OAuth/Turnstile stack? Or replicate Discord/Patreon/email login?
3. **Monetization scope** — does the fork have *any* real-money commerce? (Big legal/product call — you cannot reuse the upstream's Stripe/PayPal/Fourthwall/Discord accounts.)
4. **PseudoCoins without money** — keep the premium-currency *economy* (wallet, upgrades, consumables, time-skips) but make coins earned/granted instead of purchased?
5. **Identity richness** — do we need Discord-guild-role perk gating and linked accounts, or a flat account model?
6. **Build floor** — the game runs 100% offline with zero backend. What's the minimum first slice — just cloud saves + identity, or also live events/messages?

---

## 0.5 Decisions locked (2026-06-06)

**Q1 — Compatibility target: Rust client only.** The Rust/Dioxus client is the sole API consumer. Consequences:
- **Auth = Hanko bearer** (§6 path 1). The cookie/session + Discord/Patreon-OAuth + Turnstile stack is *not* rebuilt as the primary credential. Discord/Patreon can return later as optional *linked* accounts if we want role perks.
- **Clean redesign — the TS contract is reference only, no byte-compatibility obligation.** Mount under `/api/synergism/*`, design paths/payloads fresh, rename the misleading `stripe/` economy prefix. No cookie-compat facade, no Turnstile, no OAuth-as-login to build.

**Q2 — Monetization: RMT parked on the side (seam kept, no items yet); the fork's own items later.** Owner's words: RMT "can sit on the side for now — it's there but shows no items / not implemented yet, but it will likely come back in," and "we will not use the same RMT items as them." Consequences:
- **Keep an inert RMT seam, don't drop it.** The purchase/checkout/entitlement plumbing exists as a clean, unimplemented extension point — the UI surfaces it but shows no items / "not yet available" — so the fork can later wire its **own** processor and its **own** items without re-architecting. Design so this doesn't paint us into a corner.
- **Fork's own catalog, zero upstream coupling.** None of the upstream's PseudoCoin items, Stripe products, PayPal clientId, NOWPayments, Fourthwall store, or Discord role IDs carry over.
- **The economy layer (wallet / upgrades / consumables / WS) is the fork's to design** with new content. Whether coins are *earnable in-game now* or the economy stays dormant until RMT returns is a follow-up — but it uses new items either way.

**Strategic frame: this is a divergent fork, not a re-host.** The Rust rewrite is the foundation for *expanding* the game. Parity with the TS reference is the **porting milestone**, not the end state. Design the API for **extension** (new currencies, items, events, content) — proprietary and forward-looking, with the TS surface as a starting reference rather than a target to clone.

**Still open (lighter, non-blocking):** identity richness (membership tiers + Discord-role perks vs. a flat "logged-in" account — Q5); which slice to build first (Q6); cloud-save quota policy; Neon project + S3/R2 bucket placement.

---

## 1. How this was derived & confidence

| Source | What it gives | Confidence |
|---|---|---|
| Client `fetch`/`WebSocket` call sites (`legacy/original/src/**`) | Method, path, request body, auth flags, when-called, success/failure handling, the TS type the response is parsed into | **High** (it's the literal code) |
| MSW mock handlers (`src/mock/**`, `mock/browser.ts`, `mock/handlers/*`, `mock/websocket.ts`) | Concrete response JSON shapes & example values, status codes | **High where present** (19–20 routes mocked) |
| Client Zod validators (esp. WS schema `Login.ts:107-188`, `buyUpgradeSchema`) | The *binding* inbound contract — client throws on mismatch | **Highest** (stricter than the mocks; build to these) |
| No mock + only client parse | Shape is **INFERRED** from how the client reads it | Medium — flagged inline |

`legacy/original/src/` and `legacy/core_split/packages/web_ui/src/` are **byte-identical** for every networking file and all 7 mock files (verified). The actively-ported `core_split` did **not** strip the API surface — it carries the same calls. So this contract is current for both.

**Things only the original server knows** (the client cannot reveal them; we design them fresh): OAuth callback URLs / `state` / scopes / client-secrets; session cookie name/flags/TTL; save-id allocation; cloud-save quotas (count/size) and whether membership gates them; what `/saves/transfer` migrates from; email-verification flow; webhook fulfillment for payments.

---

## 2. Target platform: `api.lmam.tech` (lmam_api) and how Synergism slots in

From `~/github/lmam_api/README.md` + workspace inspection:

```
Request → Cloudflare Worker (apps/api)
            ├─ logging + error middleware            (api-core)
            ├─ Hanko JWT auth on /api/* → Principal   (api-core)
            └─ /api/<app>/* → routes/<app>.ts → <app>-service → <app>-db (Prisma → Neon)
```

- **Routing:** Hono, one Worker, everything under `/api/<app>/*`.
- **Data:** Prisma 7 → **Neon Postgres, one project per app** (no shared DB). Synergism gets its own Neon project + `synergism-db` package.
- **Auth:** **Hanko** (hosted auth; JWT validated via JWKS → `Principal`) wired once in `api-core`. Authenticated routes expect `Authorization: Bearer <jwt>`.
- **Layering is lint-enforced:** routes may import only their own `*-service` + `api-core`; authz + Zod validation live in the **service** layer; cross-app access is a direct service call, never a `*-db` import or HTTP hop.
- **Already present and directly reusable:**
  - **Durable Objects** — `apps/api/src/do/CampaignSession.ts` exists. This is the template for a `ConsumablesHub`/`ConsumablesSession` DO backing `wss://…/consumables/connect` (DOs are how Cloudflare Workers do stateful WebSockets + broadcast).
  - **S3 + presigner** — `@aws-sdk/client-s3` + `s3-request-presigner` are already deps. Cloud-save blobs → S3 (or Cloudflare R2 via the S3 API); metadata rows → Neon. Presigned URLs let big save blobs skip the Worker body.
  - **Env convention** — short per-app prefix (`CM_` campaigns, `TMPL_` template). Synergism → `SYN_` (`SYN_DATABASE_URL`, `SYN_SAVES_BUCKET`, `SYN_SESSION` DO binding, etc.).

**Implication:** the natural lmam_api shape mounts Synergism under `/api/synergism/*` with Hanko bearer auth — which is **not** how the legacy client calls it (root-mounted paths, cookie auth). That gap is the auth/compat decision in §6 and §10.

---

## 3. Master endpoint inventory

`%` = base URL (`https://synergism.cc` by default). **Auth**: `public` / `cookie` (session) / `cookie+` (`credentials:'include'`) / `nav` (full-page browser navigation, not fetch). **Mock**: ✓ has an MSW handler, ✗ falls through to network.

| # | Domain | Method | Path | Auth | Purpose | Mock |
|---|---|---|---|---|---|---|
| 1 | Identity | GET | `%/api/v1/users/me` | cookie | Account + subscription + quark bonus + linked accounts; 401 ⇒ logged out | ✓ |
| 2 | Identity | GET | `%/api/v1/users/logout` | cookie | Clear session, then client reloads | ✗ |
| 3 | Identity | nav | `%/login?with={discord\|patreon}[&link=true]` | — | OAuth login / account-link (new tab) | ✗ |
| 4 | Identity | POST | `%/register` | public | Email+password signup (form POST + Turnstile) | ✗ |
| 5 | Identity | POST | `%/signin` | public | Email+password login (form POST + Turnstile) | ✗ |
| 6 | Identity | POST | `%/forgot-password` | public | Reset email (Turnstile; 1/day server-enforced) | ✗ |
| 7 | Saves | GET | `%/saves/retrieve/all` | cookie | All saves **with** blob | ✓ |
| 8 | Saves | GET | `%/saves/retrieve/metadata` | cookie | Saves **without** blob (client doesn't use it yet) | ✓ |
| 9 | Saves | POST | `%/saves/upload` | cookie | Upload current save (multipart `file`+`name`) → `"Ok!"`/400 | ✓ |
| 10 | Saves | DELETE | `%/saves/delete` | cookie | Delete by `{name}` → 204 | ✓ |
| 11 | Saves | GET | `%/saves/transfer` | cookie | Migrate saves from "old system" → `"Ok!"` | ✓ |
| 12 | Live | GET | `%/api/v1/quark-bonus` | public | `{bonus:number}` global quark % (polled 15 min) | ✓ |
| 13 | Live | GET | `%/events/get` | public | Single `GameEvent` (14 buffs + window) (polled 5 min) | ✗ |
| 14 | Live | GET | `%/contributors` | public | `{contributors:[{login,avatar_url}],artists:[]}` | ✗ |
| 15 | Live | GET | `%/translations/{lang}.json` | public | i18n fallback (primary is a same-origin static file) | ✗ |
| 16 | Messages | GET | `%/messages/unread` | cookie+ | `{success,data:Message[]}` in-game news | ✓ |
| 17 | Messages | POST | `%/messages/{id}/mark-read` | cookie+ | `{success}` / 404 | ✓ |
| 18 | Economy | GET | `%/stripe/coins` | cookie | `{coins:number}` PseudoCoin wallet balance | ✓ |
| 19 | Economy | GET | `%/stripe/upgrades` | cookie | `{coins,upgrades[],playerUpgrades[]}` upgrade shop + owned | ✓ |
| 20 | Economy | PUT | `%/stripe/buy-upgrade/{id}` | cookie | Spend coins → `{upgradeId,level}` | ✓ |
| 21 | Economy | GET | `%/consumables/list` | public | `ConsumableListItem[]` shop catalog | ✓ |
| 22 | Commerce | GET | `%/stripe/products` (+`/stripe/test/products`) | public | `Product[]` (coin packs + subs) | ✓ |
| 23 | Commerce | POST | `%/stripe/create-checkout-session` (+`/test/`) | cookie | Cart → `{redirect}` Stripe Checkout URL | ✗ |
| 24 | Commerce | POST | `%/now-payments/checkout` | cookie | Cart → `{redirect}` crypto invoice URL | ✗ |
| 25 | Commerce | POST | `%/paypal/orders/create` | cookie | One-time order → `{id}` | partial |
| 26 | Commerce | POST | `%/paypal/orders/{id}/capture` | cookie | Capture order → PayPal capture obj | ✗ |
| 27 | Commerce | POST | `%/paypal/subscriptions/create?product=` | cookie | → `{id}` | ✓ |
| 28 | Commerce | POST | `%/paypal/subscriptions/revise?product=` | cookie | Upgrade/downgrade → `{link}` | ✓ |
| 29 | Commerce | POST | `%/paypal/subscriptions/cancel` | cookie | → 204 | ✓ |
| 30 | Commerce | GET/POST | `%/stripe/manage-subscription`, `%/stripe/subscription/{upgrade,downgrade,cancel}` (+`/test/`) | cookie | Stripe billing portal / tier change → `{link}` | ✗ |
| 31 | Commerce | GET | `%/merch/products` | public | `MerchProduct[]` (Fourthwall passthrough) | ✓ |
| — | Economy | **WS** | `wss%/consumables/connect` | cookie-on-upgrade | Live consumables: inventory, activation, broadcasts, tips, lotus, time-skips | ✓ |

Cross-cutting facts worth preserving (or deliberately discarding): only `quark-bonus`, `users/me`, `users/logout` sit under `/api/v1/`; **everything else is root-mounted** (`/events/get`, `/saves/*`, `/messages/*`, `/stripe/*`, …). The `stripe/` prefix on the **wallet/upgrade** endpoints (18–20) is a misnomer — those spend *already-owned* coins and touch no card. Money units differ: Stripe `price` is **cents**, Fourthwall `unitPrice.value` is **dollars**. `prod` builds drop the `/test` segment; dev builds insert it (so the old server serves both live + test Stripe).

---

## 4. Per-domain detail

### 4.1 Identity / Auth (endpoints 1–6)

**Auth model (today):** pure **cookie/session**. No bearer token, no CSRF header anywhere (grep-verified). `/me` uses `credentials:'same-origin'`; logout/saves rely on the browser default; messages use `credentials:'include'`. Login is **not** a fetch — it's OAuth redirects (`/login?with=…`) and HTML form POSTs (`/register`, `/signin`, `/forgot-password`), each guarded by a **Cloudflare Turnstile** captcha. Providers observable in the client: **Discord** and **Patreon** (OAuth), **email/password**; `steam` appears as an `accountType` for the deferred desktop build (no web `/login?with=steam`).

**`GET /api/v1/users/me` → `SynergismUserAPIResponse`** is the spine of the whole logged-in experience:
```ts
{
  member: AccountMetadata[accountType],   // Discord guild-member | {email,verified} | Patreon | {steam...} | null
  accountType: 'discord'|'patreon'|'email'|'steam'|'none',
  bonus: { quark: number },               // personal quark bonus %
  subscription: { provider:'paypal'|'stripe'|'patreon'|'steam', tier:number, endDate:string } | null,
  linkedAccounts: string[],               // subset of ['discord','patreon','email','steam']
  error?: unknown
}
```
- `accountType:'none'` + `member:null` (HTTP 401) is the "logged out" signal.
- For Discord accounts the `member.roles[]` array carries the live **Synergism Discord guild roles**, and the client maps **hardcoded role IDs** → membership tiers and event-bonus perks (`Login.ts:349-411`). These IDs belong to the *upstream* guild (see §8).
- `subscription` + `bonus.quark` are the entitlement read-path the commerce flows depend on.

**Freedom to change:** very high. Because the Rust client is new, we can replace this entire stack with Hanko bearer auth + a Synergism profile service. The only piece with gameplay teeth is the *shape* of `users/me` (subscription tier + quark bonus + linked accounts feed real bonuses) — we can redesign the wire format but must preserve those *concepts* if we keep memberships/perks.

### 4.2 Cloud Saves (endpoints 7–11)

**Auth:** cookie, same-origin; wired only when logged in. **Blob codec (load-bearing):** the stored `save` string is `base64(gzip(localStorage['Synergysave2']))`, where the localStorage value is itself the game's base64 export. Upload sends the *plain* base64 export as a multipart `file` (+ `name`); the server gzips+rebase64s it; download/load invert exactly that. The uploaded payload must be **ASCII-only** (mock rejects bytes >127 with 400). Delete keys off **`name`**, not `id` (implies per-user name uniqueness). No client-side size/slot/quota limit — any quota is server-only, and the upload error path returns **plain-text** shown directly to the user (the natural place to report "quota exceeded" / "membership required").

**`Save = { id:number, name:string, uploadedAt:string, save:string }`.** `/saves/retrieve/metadata` is the same minus `save` (exists server-side, unused by the current client — a free optimization for the Rust client: list without downloading every blob). `/saves/transfer` migrates from an unspecified "old system" using only the session identity.

**Freedom to change:** high. The Rust port has a **fresh save format** (per CLAUDE.md — no TS-save compatibility), so the blob codec and even the upload mechanism are ours to redesign. Natural redesign: presigned-S3 PUT for the blob + a metadata row in Neon; list via metadata; soft per-user quota by membership tier. The *only* concept to keep is "named save slots per user."

### 4.3 Live game services (endpoints 12–15)

All **public** (no auth), all read-only, all fail-soft:

- **`GET /api/v1/quark-bonus` → `{bonus:number}`** — a site-wide quark **percentage** (mock `105.3`). Polled every 15 min. Combined multiplicatively with the personal bonus from `users/me`. Fails open (keeps last value / 0). This is the HTTP read-model of the global event/Happy-Hour-Bell state that the WS mutates.
- **`GET /events/get` → `GameEvent`** — a **single** global event object (not a list). 14 numeric buff fields (`quark, goldenQuark, cubes, powderConversion, ascensionSpeed, globalSpeed, ascensionScore, antSacrifice, offering, obtainium, octeract, blueberryTime, ambrosiaLuck, oneMind`) as **additive fractions** (`0.25` = +25%), plus `start`/`end` epoch-ms and `name[]`/`url[]`/`color[]` arrays. "Active" iff `now ∈ [start,end]` **and** `name.length>0`. Polled every 5 min; fails closed to "no event." **No mock exists** — schema is from the `GameEvent` TS interface. (Note: maps directly onto the Rust `Event`/`BuffType` porting work and the known global-speed parity bug — same 14 buffs.)
- **`GET /contributors` → `{contributors:[{login,avatar_url}],artists:string[]}`** — GitHub-style; likely a cached proxy of the repo contributors. Fetched once on the credits screen. No mock.
- **`GET /translations/{lang}.json`** — **primary path is a same-origin static asset**; the API endpoint is only a cross-origin fallback. Langs: `en,zh,fr,de,pl,es,ru`. In this repo, `assets/translations/en.json` is the canonical source. This is essentially a static-file concern, not a real API.

**Freedom to change:** total. These are trivial to re-host (events + quark-bonus are a couple of admin-editable rows; contributors is a cron-cached GitHub fetch; translations is static hosting). Good candidate for the **first** backend slice because there's no auth and no money.

### 4.4 Messages / news (endpoints 16–17)

In-game news cards. `GET /messages/unread` → `{success, data:Message[]}` where `Message = {id,title,content,type:'info'|'warning'|'error'|'success',priority:number,is_active:boolean,created_at,updated_at,expires_at?}`. Server filters to active+unread-for-user; client sorts by `priority` desc. `POST /messages/{id}/mark-read` records per-user read state (cookie auth, `credentials:'include'`). Fails closed (empty list / `false`), never throws. **Freedom: high** — straightforward CRUD + per-user read-state join; redesign at will.

### 4.5 PseudoCoins economy + commerce (endpoints 18–31 + WS)

Two layers, **separate decisions**:

**(a) The economy (processor-independent): endpoints 18–21 + the WS.** Wallet balance (`/stripe/coins`), the upgrade shop and owned levels (`/stripe/upgrades` → `playerUpgrades[]`), spending coins on upgrades (`PUT /stripe/buy-upgrade/{id}`), the consumables catalog (`/consumables/list`), and the live consumables WS. `PseudoCoinUpgrades.ts` maps each `internalName`+level to a concrete gameplay effect (e.g. `CUBE_BUFF` → `1 + 0.06·level`; `INSTANT_UNLOCK_1/2`, slot-QOL upgrades, offering/obtainium/ambrosia/red buffs). **This layer has gameplay teeth** — if the fork keeps PseudoCoins at all, this must exist, but it needs *no payment processor* (only a source of coins).

**(b) Real-money checkout (processor-coupled): endpoints 22–31.** Stripe Checkout, NOWPayments crypto, PayPal one-time + subscriptions, Stripe billing-portal/subscription management, and a read-only **Fourthwall** merch gallery. Fulfillment is **webhook-driven and asynchronous** — the client never gets coins in a response; it **polls** `/stripe/coins` (PayPal capture uses an exponential balance-check at `[15,30,60,120,180,240,300]s`) or re-reads `users/me` for subscription changes. The client ships the upstream's **public PayPal clientId** and points at the upstream's Stripe catalog + Fourthwall store.

**Freedom to change:** layer (a) is ours to redesign freely (rename the misleading `stripe/` prefix, restructure the upgrade catalog). Layer (b) is a **product/legal decision, not a technical one** — see §8.

---

## 5. The consumables WebSocket (`wss://…/consumables/connect`)

The one piece of real-time infrastructure. Opened **only when logged in**, authenticated by the **session cookie on the WS upgrade** (no token/subprotocol). Reconnects with backoff `[5s,15s,30s,60s]` then gives up; no app-level heartbeat. Build the server to the **client's Zod schema (`Login.ts:107-188`)**, which is stricter than the mock in three places (the real server *must* send fields the mock omits).

**Client → server (4):**
| type | shape | when |
|---|---|---|
| `consume` | `{type,consumable:string,version:'2'}` | buy/activate a Bell, Timeskip, or Lotus pack |
| `applied-tip` | `{type,amount:int}` | redeem N offline-time tips |
| `applied-lotus` | `{type,amount:1}` | consume a lotus in the Anthill |
| `confirm` | `{type,id:uuid,consumableId:string}` | ACK a received time-skip (at-least-once handshake) |

**Server → client (15):** `join`, `error`(+resets socket), `warn`, `consumed{consumable,displayName,startedAt}` (broadcast; 1h buff), `consumable-ended{consumable,name}` (broadcast), `info-all{active[],tips,inventory[]}` (snapshot on connect), `thanks` (to actor; triggers coin refresh), `tips{tips}`, `tip-backlog{tips}` (on reconnect), `applied-tip{amount,remaining}`, `time-skip{consumableName,id:uuid,amount}` (client applies + `confirm`s), `lotus{consumableName,amount}`, `applied-lotus{remaining,lifetimePurchased}`, `lotus-ended`, `lotus-active{remainingMs}`.

**Semantics:** three families keyed by `internalName` substring — **`HAPPY_HOUR_BELL`** (global broadcast buff, also pays the activator 12h of tips; this is what spikes `/api/v1/quark-bonus`), **`*_TIMESKIP`** (per-user fast-forward with the `confirm` reliability handshake), **`LOTUS_*`** (per-user timed inventory). **The "buy a consumable" transaction happens over the WS, not HTTP** — the catalog is HTTP (`/consumables/list`), the spend + grant is WS.

**lmam_api fit:** a Durable Object (`ConsumablesHub`) holds connections + global Bell state and fan-outs `consumed`/`consumable-ended`; per-user DO state (or Neon rows) holds tips/lotus/pending-timeskips. The existing `CampaignSession` DO is the working pattern. **Freedom:** if the fork drops paid consumables, this whole subsystem and its DO are deferrable — the game is fully playable without it.

---

## 6. Auth: the central integration question

This is the biggest gap between "what the client does" and "what lmam_api does," and the place where "leave the option to change it" matters most.

| | Legacy Synergism client | lmam_api today |
|---|---|---|
| Credential | HttpOnly **session cookie** | **Hanko JWT** (`Authorization: Bearer`) |
| Login UX | Discord/Patreon OAuth redirects + email/pw form POST + Turnstile | Hanko hosted (passkeys/email/social) |
| Logout | `GET /api/v1/users/logout` clears cookie | token expiry / Hanko |
| CSRF | none (cookie + no header) | bearer ⇒ N/A |
| WS auth | cookie on upgrade | (browsers can't set WS headers → token via query/subprotocol) |
| Identity payload | rich `users/me` (Discord member, roles, sub, bonus, links) | minimal `Principal` |

**Three viable paths:**
1. **Adopt Hanko (recommended default).** New Rust client sends `Authorization: Bearer`. Build a `synergism-service` "profile" that augments the Hanko `Principal` with game data (subscription tier, quark bonus, linked Discord/Patreon, save quota). Discord/Patreon become *linked accounts* (via Hanko social connectors or a custom OAuth-link flow stored in `synergism-db`), not the primary credential. WS auth: pass the JWT as a query param to the DO and validate it there. **Pro:** reuses everything lmam_api already built; least work; consistent with your other apps. **Con:** the legacy TS client (cookie-based) would *not* work unmodified against it.
2. **Cookie-session shim.** Re-implement the cookie/OAuth/Turnstile contract so the *old TS client* runs as-is. **Pro:** byte-compatibility, useful if you want the existing client live during the Rust transition. **Con:** you rebuild Discord+Patreon OAuth, Turnstile, and session cookies from scratch and diverge from lmam_api's auth standard — significant, throwaway-ish work.
3. **Hybrid:** Hanko for the new client; a thin cookie-compat facade only if/when you need the old client temporarily.

Decision driver: **§10 Q1 (compatibility target)**. If the Rust client is the only consumer, path 1 is clearly right.

---

## 7. Build vs. change — per domain

| Domain | Must exist for a playable fork? | Faithful-port option | Recommended redesign | Effort |
|---|---|---|---|---|
| **Live: events + quark-bonus** | No (fails soft) but high value, low cost | Match `GameEvent`/`{bonus}` shapes | 2 admin-editable Neon tables + public GET; keep the 14 buff concepts (aligns with the Rust `Event` port) | **S** |
| **Live: contributors** | No | proxy GitHub | cron-cached GitHub fetch → 1 row/blob | **XS** |
| **Live: translations** | No (static-first) | static file + fallback | static hosting (R2/Pages); maybe drop the API fallback | **XS** |
| **Messages/news** | No | match `Message` shape | CRUD + per-user read-state; redesign freely | **S** |
| **Identity** | Only if saves/social/commerce exist | replicate cookie+OAuth | **Hanko + profile service** (§6 path 1) | **M** |
| **Cloud saves** | High-value, optional | match blob codec + multipart | presigned-S3 blob + Neon metadata + tier quota; fresh format | **M** |
| **PseudoCoins economy** (wallet/upgrades/consumables) | Only if fork keeps premium currency | match `/stripe/*` economy + WS | rename off `stripe/`; coins **earned/granted** not bought (Q4); DO for WS | **L** (WS is the cost) |
| **Real-money commerce** | No | rebuild on your *own* processor accounts | **defer or drop** (see §8) | **XL / legal** |
| **Merch** | No | Fourthwall passthrough | drop, or point at your own store | **XS / N/A** |

---

## 8. Fork red flags (read before touching commerce)

These are not technical blockers — they're ownership/legal/ethical constraints a fork inherits:

- **You cannot reuse the upstream's payment accounts.** The client embeds the upstream project's **public PayPal clientId**, points at the upstream's **Stripe** price catalog + billing portal, the upstream's **NOWPayments** merchant, and the upstream's **Fourthwall** storefront + CDN. Any real-money commerce in a fork requires *your own* Stripe/PayPal/NOWPayments/Fourthwall accounts and webhook fulfillment, built from scratch (and several of these routes have no mock, so even the response shapes are inferred).
- **Discord role IDs belong to the upstream guild.** The hardcoded tier/event-bonus role IDs (`Login.ts:349-411`) reference the official Synergism Discord. A fork has a different guild (or none); the role→perk mapping must be re-pointed or removed.
- **Monetizing a fork of someone else's game is a product/legal decision, not an engineering one.** Synergism is open-source, but its revenue infrastructure (memberships, PseudoCoins-for-money, merch) is the upstream author's livelihood. Recommendation: **default the fork to zero real-money commerce.** If you still want the PseudoCoins *gameplay*, make coins **earned/granted** (Q4) so the whole processor layer (22–31) disappears.
- **`VITE_API_BASE_URL` must be re-pointed.** Until set to your backend, the client hammers `synergism.cc` — i.e. the upstream's production server. The fork must build with its own base URL (and `wsBaseUrl`, `canonicalHost`).

---

## 9. Suggested phased roadmap (sequenced by value ÷ cost, money last)

- **Phase 0 — none.** Game runs fully offline/local (current state). Confirm the Rust client needs *nothing* to be playable; the backend is additive.
- **Phase 1 — Live services (no auth).** `events`, `quark-bonus`, `contributors`, translations hosting. Stands up the `synergism` app in lmam_api, proves Hono+Neon wiring, zero auth/money risk. Directly supports the Rust `Event` port.
- **Phase 2 — Identity (Hanko) + Cloud saves.** Hanko bearer + `synergism` profile service; presigned-S3 saves + Neon metadata + tier-less quota. This is the first *logged-in* slice and the highest-value optional feature.
- **Phase 3 — Messages/news.** Cheap, pleasant, needs Phase 2 auth for read-state.
- **Phase 4 — PseudoCoins economy *without money* (only if desired).** Wallet + upgrades + consumables + the WS Durable Object, with coins earned/granted. Big lift (the DO), no processors.
- **Phase 5 — Real-money commerce (only if the fork decides to, with its own accounts).** Stripe/PayPal/etc. Most likely **never** for a fork; gated entirely on §8 + Q3.

---

## 10. Open decisions for you

> **Resolved 2026-06-06 (see §0.5):** Q1 = *Rust client only* → Hanko auth + clean redesign. Monetization = *RMT parked as an inert seam*, the fork's own items later (Q3/Q4 fold into this). **Remaining open:** Q5 (identity richness), Q6 (first slice), and the smaller ones below.

1. **Compatibility target.** Is the **Rust/Dioxus client the only consumer** (→ free redesign, Hanko auth, clean paths), or must the **legacy TS client keep working** against the new API for a transition (→ byte-compatible paths + cookie/OAuth shim)? *This one gates almost everything else.*
2. **Auth.** Adopt **Hanko** (recommended) and treat Discord/Patreon as optional *linked* accounts? Or replicate the upstream's Discord+Patreon+email+Turnstile login?
3. **Real-money commerce.** Does the fork have **any** paid transactions? (Default recommendation: **no** — drop endpoints 22–31 + merch.)
4. **PseudoCoins.** Keep the premium-currency **gameplay** (wallet/upgrades/consumables/time-skips) with coins **earned/granted** instead of bought? Or cut PseudoCoins entirely (also removes the WebSocket)?
5. **Identity richness.** Do we want membership **tiers + Discord-role perks + linked accounts**, or a **flat account** (just "logged in," for saves)?
6. **First slice.** Start with **Phase 1 (live services, no auth)** to stand the app up, or jump straight to **Phase 2 (identity + cloud saves)** as the first user-visible win?

Plus smaller ones to confirm later: cloud-save **quota** policy (count/size, tier-gated?); where this app's **Neon project** + **S3/R2 bucket** live; whether `/api/synergism/*` mounting is acceptable to the Rust client (it is — we control the client) or we want root-mounted paths.

---

## Appendix — key data shapes (reference)

```ts
// users/me
interface SynergismUserAPIResponse {
  member: DiscordMember | { email:string; verified:boolean } | PatreonUser | SteamUser | null
  accountType: 'discord'|'patreon'|'email'|'steam'|'none'
  bonus: { quark: number }                 // personal %, combined with global quark-bonus
  subscription: { provider:'paypal'|'stripe'|'patreon'|'steam'; tier:number; endDate:string } | null
  linkedAccounts: string[]
  error?: unknown
}

// events/get  (buffs are additive fractions; "active" iff now∈[start,end] && name.length>0)
interface GameEvent {
  name:string[]; url:string[]; color:string[]; start:number; end:number  // epoch ms
  quark:number; goldenQuark:number; cubes:number; powderConversion:number
  ascensionSpeed:number; globalSpeed:number; ascensionScore:number; antSacrifice:number
  offering:number; obtainium:number; octeract:number; blueberryTime:number
  ambrosiaLuck:number; oneMind:number
}

// cloud save  (save = base64(gzip(localStorage export)); ASCII-only on upload; delete by name)
interface Save { id:number; name:string; uploadedAt:string; save:string }

// messages/unread
interface Message {
  id:number; title:string; content:string
  type:'info'|'warning'|'error'|'success'; priority:number; is_active:boolean
  created_at:string; updated_at:string; expires_at?:string
}

// economy
type CoinBalance = { coins:number }
type Upgrade = { upgradeId:number; maxLevel:number; name:string; description:string
                 internalName:PseudoCoinUpgradeName; level:number; cost:number }
type UpgradesResponse = { coins:number; upgrades:Upgrade[]; playerUpgrades:OwnedUpgrade[] }
type ConsumableListItem = { name:string; description:string; internalName:string; length:string; cost:number }
```

*(Full field-by-field contracts, mock example payloads, and the 16 PseudoCoin upgrade `internalName`s live in the four audit transcripts that produced this doc; reproduce on request.)*
