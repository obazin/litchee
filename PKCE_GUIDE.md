# Logging in with Lichess: a step-by-step PKCE guide

This guide walks you through **"Log in with Lichess"** using `litchee`, from
zero. It is written for beginners — no prior OAuth knowledge is assumed. By the
end you will have an access token you can use to call the Lichess API on a
user's behalf.

---

## What problem are we solving?

You want your app to act on behalf of a Lichess user — read their account, play
games, create challenges, etc. You **could** ask them to paste a personal access
token, but that is clumsy and dangerous. Instead, Lichess lets the user click a
button, log in on lichess.org, and approve your app. Your app then receives a
token. This is the **OAuth2 Authorization Code flow**.

### Why "PKCE"?

The classic OAuth2 flow assumes your app can keep a **client secret** hidden on a
server. But many apps can't: a mobile app, a single-page web app, or a desktop
CLI all ship their code to the user, so any embedded secret is exposed.

**PKCE** (pronounced "pixy", short for *Proof Key for Code Exchange*) fixes this
**without a secret**. The idea, in one sentence:

> Before sending the user off to log in, your app invents a random secret
> (the **verifier**), sends only a *hash* of it (the **challenge**) to Lichess,
> and later proves it owns the original by sending the verifier back.

Because only your app ever knows the original verifier, an attacker who
intercepts the redirect can't complete the login. `litchee` generates and hashes
the verifier for you, so you never compute a SHA-256 by hand.

---

## The flow at a glance

```
   YOUR APP                         LICHESS                      USER
      │                                │                          │
      │ 1. build authorization URL     │                          │
      │    (litchee makes verifier     │                          │
      │     + challenge + state)       │                          │
      │                                │                          │
      │ 2. send user to that URL ──────┼────────────────────────► │
      │                                │   user logs in & approves │
      │                                │                          │
      │ ◄──────────────────────────────  3. redirect back with    │
      │     code + state                  ?code=...&state=...      │
      │                                │                          │
      │ 4. verify state matches        │                          │
      │                                │                          │
      │ 5. exchange code + verifier ──►│                          │
      │ ◄────────────── access token ──│                          │
      │                                │                          │
      │ 6. use token to call the API   │                          │
```

Three things `litchee` hands you in step 1 and you must **keep until step 5**:

| Thing      | What it is                              | Why you keep it                          |
|------------|-----------------------------------------|------------------------------------------|
| `url`      | Where to send the user                  | You redirect/open the browser to it      |
| `state`    | A random anti-forgery string            | You compare it to what comes back (CSRF) |
| `verifier` | The PKCE secret (`PkceVerifier`)        | You need it to exchange the code         |

---

## Before you start

You need to decide two things up front. **Neither is a secret** — both are
public and chosen by you.

1. **`client_id`** — any string that identifies your app, e.g.
   `"com.example.my-chess-app"`. You make it up; there is no registration step
   on Lichess.
2. **`redirect_uri`** — the absolute URL Lichess sends the user back to after
   they approve. It must match exactly in steps 1 and 5.
   - Web server: `https://myapp.example.com/callback`
   - Local dev / desktop: `http://localhost:8080/callback`

You also pick the **scopes** — the permissions you are requesting. Ask for the
least you need. Examples (`litchee::api::auth::oauth::Scope`):

| Scope                  | Lets you…                          |
|------------------------|------------------------------------|
| `Scope::PreferenceRead`| read the user's preferences        |
| `Scope::ChallengeWrite`| create and accept challenges       |
| `Scope::BoardPlay`     | play games via the Board API       |
| `Scope::BotPlay`       | play games as a bot                |
| `Scope::EmailRead`     | read the user's email address      |

> An empty scope list is allowed and gives you only public, read-only access.

---

## Step 1 — Build the authorization URL

> ### What does "build the URL" mean?
> It is easy to misread "build the URL" as "**open** the URL" or "**send a
> request** to it". It does **not** mean that. Here, *build* means **construct
> the URL string** — assemble a `https://lichess.org/oauth?...` address with all
> the right query parameters glued on. Nothing is sent anywhere in this step; no
> network call happens. You are just *creating an address*.
>
> Think of it like writing down a destination on an envelope. You are not mailing
> anything yet — you're only preparing where it will eventually go. Your app
> never visits this URL itself; in [Step 2](#step-2--send-the-user-to-that-url)
> you hand it to the *user's browser*, which is what actually opens it.
>
> **How is it built?** A valid authorization URL needs several query parameters
> in a specific format — `response_type`, `client_id`, `redirect_uri`,
> `code_challenge`, `code_challenge_method=S256`, `scope`, and `state`. Getting
> any of them wrong (especially the PKCE challenge, which is a SHA-256 hash,
> base64-url encoded) breaks the flow. So **you don't build the string by hand** —
> you fill in an `AuthorizationRequest` with your plain inputs, and
> `client.oauth().authorization_url(...)` does the assembling, hashing, and
> URL-encoding for you. The "building" is `litchee`'s job; your job is to supply
> the inputs and then use the result.

Create a client (no token yet — we're trying to *get* one) and ask `litchee` to
build (assemble) the URL.

```rust
use litchee::LichessClient;
use litchee::api::auth::oauth::{AuthorizationRequest, Scope};

let client = LichessClient::builder()
    .user_agent("my-chess-app/1.0")
    .build()?;

let request = AuthorizationRequest {
    client_id: "com.example.my-chess-app",
    redirect_uri: "http://localhost:8080/callback",
    scopes: &[Scope::PreferenceRead, Scope::ChallengeWrite],
    username_hint: None, // or Some("thibault") to pre-fill the login form
};

let authorization = client.oauth().authorization_url(&request)?;

// You now have three values to use/keep:
println!("Send the user here:\n{}", authorization.url);
let state    = authorization.state;     // keep this
let verifier = authorization.verifier;  // keep this (it's the PKCE secret)
```

`authorization.url` already contains the `code_challenge`, `code_challenge_method=S256`,
`state`, `scope`, and your `client_id`/`redirect_uri`. You do not assemble any of
that yourself.

---

## Step 2 — Send the user to that URL

How you do this depends on your app:

- **Web app:** respond with an HTTP redirect (`302 Found`) to `authorization.url`.
- **Desktop / CLI:** open the system browser, e.g. with the [`open`](https://crates.io/crates/open)
  crate, or just print the URL and ask the user to paste it.
- **Mobile:** open it in an in-app browser tab.

The user now logs in on lichess.org and approves (or denies) your requested
scopes. **Lichess never sees your verifier — only its hash.**

> ### Where do I keep `state` and `verifier`?
> They must survive until the redirect comes back.
> - **Web server:** store them in the user's session (keyed by a cookie).
> - **Desktop/CLI:** keep them in memory in the same running process — the
>   simplest case, since you control the whole flow.
>
> The `PkceVerifier`'s `Debug` output is redacted, so it won't leak into logs by
> accident. Don't write it to disk or send it anywhere.

---

## Step 3 — Receive the redirect

When the user approves, Lichess redirects the browser to your `redirect_uri`
with two query parameters:

```
http://localhost:8080/callback?code=abc123...&state=THE_SAME_STATE
```

- `code` — a short-lived **authorization code** (not yet a token).
- `state` — should equal the `state` you saved in step 1.

If the user **denies**, you instead get `?error=access_denied` — handle that as a
normal "user said no" outcome, not a crash.

> **Desktop/CLI tip:** to catch the redirect on `http://localhost:8080`, run a
> tiny one-request HTTP server that reads the query string, then shuts down. Any
> minimal HTTP server crate works.

---

## Step 4 — Verify `state` (don't skip this!)

This single check is what protects you against cross-site request forgery. If
the returned `state` doesn't match what you stored, **abort** — the request may
be forged.

```rust
let returned_state = /* the `state` query param from the redirect */;

if returned_state != state {
    // Reject: do NOT exchange the code.
    return Err("state mismatch — possible CSRF, aborting".into());
}
```

---

## Step 5 — Exchange the code for a token

Now prove you own the verifier. Pass the `code` from the redirect plus your saved
`verifier`. **`redirect_uri` and `client_id` must match step 1 exactly**, or
Lichess rejects the exchange.

```rust
use litchee::api::auth::oauth::CodeExchange;

let returned_code = /* the `code` query param from the redirect */;

let exchange = CodeExchange {
    code: returned_code,
    code_verifier: &verifier,                       // the secret from step 1
    redirect_uri: "http://localhost:8080/callback", // same as step 1
    client_id: "com.example.my-chess-app",          // same as step 1
};

let token = client.oauth().exchange_code(&exchange).await?;

println!("Got a token! It expires in {} seconds.", token.expires_in);
// token.access_token holds the secret bearer string (Debug-redacted).
```

On success you get a [`LichessToken`] with:
- `access_token` — the bearer string you authenticate with,
- `token_type` — always `"Bearer"`,
- `expires_in` — lifetime in seconds (Lichess tokens are long-lived, ~1 year).

If something is wrong (e.g. a mismatched verifier or a reused code), `litchee`
returns a typed `LichessError::OAuth` you can match on — for example the
`invalid_grant` case.

---

## Step 6 — Use the token

Build a **new** client carrying the token, and call the API:

```rust
let user_client = LichessClient::builder()
    .user_agent("my-chess-app/1.0")
    .token(token.access_token)
    .build()?;

// Now authenticated calls work, e.g.:
let me = user_client.account().profile().await?;
println!("Logged in as {}", me.user.username);
```

Store `access_token` securely if you need it across restarts (OS keychain,
encrypted store, server-side session) — treat it like a password.

---

## Step 7 (optional) — Log out / revoke

When the user logs out, revoke the token so it can't be reused:

```rust
user_client.oauth().revoke_token().await?;
```

---

## Common mistakes & how to avoid them

| Symptom                                   | Likely cause                                              | Fix                                              |
|-------------------------------------------|-----------------------------------------------------------|--------------------------------------------------|
| `invalid_grant` on exchange               | `redirect_uri` or `client_id` differs from step 1         | Make all three params byte-for-byte identical    |
| `invalid_grant` / "code already used"     | Reused an authorization code, or it expired               | Each code is single-use & short-lived; start over|
| State mismatch every time                 | You didn't persist `state` across the redirect            | Save it in the session / process before step 2   |
| Lost the verifier after redirect          | Stored it somewhere that didn't survive the round trip    | Keep it with `state` in the same session         |
| Token works for some calls, 403 on others | Missing scope                                             | Request the needed `Scope` in step 1             |

---

## The whole thing, end to end

For a desktop/CLI app that owns the full flow in one process:

```rust
use litchee::LichessClient;
use litchee::api::auth::oauth::{AuthorizationRequest, CodeExchange, Scope};

const CLIENT_ID: &str = "com.example.my-chess-app";
const REDIRECT_URI: &str = "http://localhost:8080/callback";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LichessClient::builder()
        .user_agent("my-chess-app/1.0")
        .build()?;

    // 1. Build the URL + keep state & verifier.
    let auth = client.oauth().authorization_url(&AuthorizationRequest {
        client_id: CLIENT_ID,
        redirect_uri: REDIRECT_URI,
        scopes: &[Scope::PreferenceRead],
        username_hint: None,
    })?;

    // 2. Send the user there (open browser / print the URL).
    println!("Open this URL and approve:\n{}", auth.url);

    // 3. Receive the redirect on localhost:8080 and read `code` + `state`.
    let (returned_code, returned_state) = wait_for_redirect().await?; // your server

    // 4. Verify state.
    if returned_state != auth.state {
        return Err("state mismatch — aborting".into());
    }

    // 5. Exchange for a token.
    let token = client.oauth().exchange_code(&CodeExchange {
        code: &returned_code,
        code_verifier: &auth.verifier,
        redirect_uri: REDIRECT_URI,
        client_id: CLIENT_ID,
    }).await?;

    // 6. Use it.
    let user_client = LichessClient::builder()
        .user_agent("my-chess-app/1.0")
        .token(token.access_token)
        .build()?;
    let me = user_client.account().profile().await?;
    println!("Logged in as {}", me.user.username);

    Ok(())
}
```

`wait_for_redirect` is the small local HTTP listener you write to capture the
`code` and `state` query parameters — that part is yours, everything else is
`litchee`.

---

## Recap

1. **Build** the authorization URL — `litchee` makes the verifier, challenge, and state.
2. **Send** the user to it.
3. **Receive** the redirect with `code` + `state`.
4. **Verify** `state` matches.
5. **Exchange** `code` + verifier for a token.
6. **Use** the token in a new client.
7. **Revoke** it on logout.

The golden rules: keep the **verifier** and **state** safe until the exchange,
always **check state**, and make `client_id` + `redirect_uri` **identical** in
steps 1 and 5.

---

## Glossary

New to OAuth? Here are the terms used in this guide, in plain language.

**OAuth2** — The industry-standard protocol for letting one app act on a user's
behalf in another service *without* the user handing over their password. "Log in
with Google/GitHub/Lichess" buttons are all OAuth2.

**Authorization Code flow** — The specific OAuth2 variant used here: the user
approves your app, you receive a temporary **authorization code**, and you swap
that code for a token. The "two-step handoff" is what makes it safe.

**PKCE** (*Proof Key for Code Exchange*, pronounced "pixy", RFC 7636) — An
add-on to the Authorization Code flow that removes the need for a client secret.
Your app invents a random secret and sends only its hash up front, then proves
ownership later. Designed for apps that can't keep a secret (mobile, desktop,
single-page web).

**Client** — Your application. (Not to be confused with the *user*.) In this
guide, also the `LichessClient` you build with `litchee`.

**`client_id`** — A public string that identifies your app. You choose it; there
is no registration step on Lichess. Not a secret.

**Client secret** — A password-like string that *server-side* OAuth apps use to
authenticate themselves. PKCE exists precisely so you **don't** need one — this
guide never uses a client secret.

**Authorization server** — The service that logs the user in and issues tokens.
Here, that's lichess.org.

**Resource server** — The API that accepts the token and serves protected data.
Here, also Lichess (the `/api/...` endpoints).

**Scope** — A single named permission you request, e.g. `board:play` or
`preference:read`. The token you receive is limited to the scopes the user
approved. Ask for the least you need.

**`redirect_uri`** (a.k.a. *callback URL*) — The absolute URL Lichess sends the
user back to after they approve. Must match **exactly** between building the URL
and exchanging the code.

**Authorization URL** — The `https://lichess.org/oauth?...` address you send the
user to so they can log in and approve. Your app *constructs* it but never visits
it itself — the user's browser does.

**Authorization code** (the `code`) — A short-lived, single-use string Lichess
returns at your `redirect_uri` after approval. It is **not** a token yet; you
must exchange it. Reusing or delaying it causes `invalid_grant`.

**Code verifier** (`PkceVerifier`) — The high-entropy random secret your app
generates and keeps private. Sent only at the final exchange step to prove your
app started the flow.

**Code challenge** — The transformed (`SHA-256` hashed, base64-url encoded) form
of the verifier, sent in the authorization URL. Lichess stores it and later
checks it against your verifier. `litchee` computes it for you.

**`code_challenge_method`** — Tells the server how the challenge was derived.
This guide always uses `S256` (SHA-256), the secure option.

**`state`** — A random anti-forgery (CSRF) value your app generates, sends in the
authorization URL, and checks when the redirect returns. If it doesn't match,
abort — the request may be forged.

**CSRF** (*Cross-Site Request Forgery*) — An attack where a malicious page tricks
a logged-in user's browser into making an unwanted request. The `state` check is
what defends against it here.

**Access token** (`LichessToken`) — The bearer string you finally receive. You
send it on every authenticated API call. Treat it like a password.

**Bearer token** — A token used by simply *presenting* it (in an
`Authorization: Bearer <token>` header). Whoever holds it can use it, hence
"keep it secret".

**`expires_in`** — How many seconds the access token stays valid. Lichess tokens
are long-lived (about a year).

**Revoke** — To invalidate a token before it expires, e.g. on logout, so it can
no longer be used.
