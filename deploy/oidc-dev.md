# Pocket ID Development Setup for Oximeal

> This guide accompanies the *IdP de desarrollo* block (tasks 6.1‑6.4) and provides
> step‑by‑step instructions to launch Pocket ID locally and configure the
> necessary resources, permissions and OAuth2 clients for Oximeal.
>
> All commands are meant to be executed from the repository root (`/data/rust/oximeal`).

## 1.  Launch Pocket ID

```bash
# Ensure podman-compose is installed.  If you prefer Docker, substitute
# `podman compose` with `docker compose` in the commands below.

# Pull and start the provider in detached mode
podman compose -f deploy/compose.yaml up -d
```

> **Notes**
> - The compose file pulls **ghcr.io/pocket-id/pocket-id:v2.19.1** – the latest stable
>   release as of this writing.
> - Pocket ID opens a local SQLite database in the named volume
>   `pocket-id-data`.  It is persisted across restarts.
> - By default the service listens on port **1411** and exposes a health endpoint
>   at `/healthz`.  If you want to access Pocket ID from the host machine you
>   can add a `ports` mapping (`1411:1411`) to the service definition.
>
> To stop and remove the container run:
> 
> ```bash
> podman compose -f deploy/compose.yaml down
> ```

## 2.  Create the **API / resource** `https://oximeal.local/api`

1. Open the Pocket ID setup wizard:
   
   ```bash
echo "http://localhost:1411/setup" | xdg-open
# or open your browser manually
>```
2. Follow the wizard and select **Create new resource**.
3. Set the **Audience URI** to:
   ```
   https://oximeal.local/api
   ```
4. Note the **Resource ID**; you'll use it as `OIDC_API_AUDIENCE` below.

## 3.  Define *permission keys*

Create the permissions used by Oximeal and their respective keys:

| Permission            | Key                        |
|-----------------------|----------------------------|
| `read:recipes`        | `recipes.read`             |
| `write:recipes`       | `recipes.write`            |
| `read:plan`           | `plan.read`                |
| `write:plan`          | `plan.write`               |
| `read:shopping`       | `shopping.read`            |
| `write:shopping`      | `shopping.write`           |
| `agent:read`          | `agent.read`               |
| `agent:plan`          | `agent.plan`               |
| `agent:shopping`      | `agent.shopping`           |

Use the Pocket ID UI to create each key and associate it with the API
resource `https://oximeal.local/api`.

## 4.  Create a *user/group* test user

1. In the **Users** section of the Pocket ID UI, create a test user,
   e.g. `dev_user@example.com`.
2. In the **Groups** section create a group named, for example, `dev-users`.
3. Add the test user to the `dev-users` group.
4. **Important** – The group name you choose must match the value you set in
   `OIDC_ALLOWED_GROUPS` (described in the next step).

## 5.  Create the OAuth2 clients

### 5.1 Client *user‑delegated* (SPA / BFF)

| Parameter | Value | Notes |
|-----------|-------|-------|
| **Type** | Confidential | The BFF acts as a *confidential* client.
| **Client name** | `Oximeal SPA` |
| **Redirect URI** | `http://localhost:8080/auth/callback` |
| **Post‑logout redirect URI** | `http://localhost:8080` |
| **Allowed user groups** | `dev-users` |
| **Permission scopes** | `recipes.read recipes.write plan.read plan.write shopping.read shopping.write *` |
| **Audience** | `https://oximeal.local/api` |

After creating the client record, note the **Client ID** and **Client Secret**;
you'll reference them in the backend environment.

### 5.2 Client *M2M* (Valet)

| Parameter | Value | Notes |
|-----------|-------|-------|
| **Type** | Confidential – *Client access* (M2M) |
| **Client name** | `Oximeal Valet` |
| **Audience** | `https://oximeal.local/api` |
| **Permission scopes** | `agent.read agent.plan agent.shopping agent.write` |

Note that **public** clients are **not** allowed to obtain M2M tokens.

## 6.  Backend environment variables

Below is an example `.env` snippet for the Oximeal backend when running
locally.  Replace the placeholders with the actual UUIDs and secrets you
obtained from Pocket ID.

```dotenv
# --------- OIDC ----------
OIDC_ENABLED=true
OIDC_ISSUER=http://localhost:1411       # Pocket ID base URL
OIDC_CLIENT_ID=YOUR_SPA_CLIENT_ID        # from 5.1
OIDC_CLIENT_SECRET=YOUR_SPA_CLIENT_SECRET
OIDC_API_AUDIENCE=https://oximeal.local/api
OIDC_REDIRECT_URI=http://localhost:8080/auth/callback
OIDC_POST_LOGOUT_REDIRECT_URI=http://localhost:8080
ALLOWED_ORIGINS=http://localhost:8080       # CORS allowed origin
OIDC_ALLOWED_GROUPS=dev-users               # group created in step 4

# --------- Session ----------
SESSION_SECRET=YOUR_SESSION_SECRET          # 32+ ASCII chars
SESSION_TTL_SECS=2592000
SESSION_REFRESH_MARGIN_SECS=300
SESSION_COOKIE_SECURE=false
# HTTP en localhost: sin prefijo __Host- y con el escape explícito.
SESSION_COOKIE_NAME=oximeal_session
ALLOW_INSECURE_COOKIE=true
SESSION_ABSOLUTE_TTL_SECS=43200
```

> **Tip** – For the `SESSION_SECRET` you can reuse the same key as
> `ENCRYPTION_KEY` in Pocket ID, but it is best practice to keep them
> separate.

## 7.  HTTPS for Production

Pocket ID **requires HTTPS** on redirect URIs that are *not* `localhost`.
To satisfy this in production you need a reverse proxy that terminates TLS
and forwards traffic to Pocket ID and the Oximeal services.  Common
choices are **Caddy**, **Traefik** or **nginx**.

A minimal Caddyfile example:

```
# Caddyfile – HTTPS + reverse‑proxy

pocketid.oximeal.local {
    reverse_proxy :1411
    tls /path/to/your/fullchain.pem /path/to/your/privkey.pem
}

oximeal.local {
    reverse_proxy :8080
    tls /path/to/your/fullchain.pem /path/to/your/privkey.pem
}
```

When deploying to a remote server:

1. Generate TLS certificates (e.g. through Let’s Encrypt).  The proxy must
   expose `https://pocketid.oximeal.local` as the `APP_URL` for Pocket ID.
2. Ensure the redirect URIs in your Pocket ID clients use `https://oximeal.local`.
3. Keep the same `OIDC_CLIENT_ID`/`SECRET` and permissions – only the
   `ISSUER` and `REDIRECT_URI` values change.

> **Why does Pocket ID enforce HTTPS?**
> For security reasons it refuses to issue tokens for redirect URIs
> that are not considered *trusted* (outside of `localhost`).  This
> protects users from man‑in‑the‑middle attacks that could
> intercept the authorization code.

---

© 2026 Oximeal. All rights reserved.
