+++
id = "ed5c2b3b"
title = "apm-ui: Vite + React + shadcn/ui skeleton wired to backend"
state = "closed"
priority = 70
effort = 4
risk = 2
author = "apm"
branch = "ticket/ed5c2b3b-apm-ui-vite-react-shadcn-ui-skeleton-wir"
created_at = "2026-03-31T06:11:40.599936Z"
updated_at = "2026-04-06T06:49:12.572530Z"
+++

## Spec

### Problem

There is no frontend. The backend steps (Steps 1 and 2) will deliver an axum server on port 3000 with `GET /health` and `GET /api/tickets`, but there is nothing to load in a browser. This ticket creates the `apm-ui/` directory with a Vite + React + TypeScript + shadcn/ui scaffold, wires TanStack Query to make one `useQuery` call to `/api/tickets` that logs results to the browser console, and configures the axum server to serve the built static files at `GET /`. The page is intentionally blank. The goal is to prove the full stack — React build → axum static serving → API fetch → console output — wires together correctly before any UI is built on top of it.

### Acceptance criteria

- [x] `npm run build` in `apm-ui/` exits 0 with no TypeScript errors
- [x] `cargo run -p apm-server` (from repo root, after building the UI) serves `GET /` with HTTP 200 and `Content-Type: text/html`
- [x] Loading `http://localhost:3000/` in a browser renders a blank page with no visible content
- [x] After the page loads, the browser devtools console shows the array returned by `/api/tickets`
- [x] If `/api/tickets` returns a non-2xx response or a network error, the console shows the error but the page does not throw an unhandled exception
- [x] `cargo test --workspace` passes after the static-serving changes are added to apm-server
- [x] `apm-ui/.gitignore` excludes `node_modules/` and `dist/`

### Out of scope

- Any visible UI components or rendering of ticket data on the page (Steps 4+)
- The Zustand store (Step 4)
- Keyboard navigation or column layout (Step 4)
- A Vite dev-server proxy to the Rust backend (dev convenience; not needed to validate the stack)
- Hot-module replacement or watch mode in production
- shadcn/ui component installation — only the base setup (Tailwind, CSS variables, `cn` util) is needed
- Authentication or CORS configuration
- Embedding the UI assets into the Rust binary at compile time

### Approach

**Prerequisite:** Step 2 (`GET /api/tickets` and `GET /api/tickets/:id`) must be `implemented` before this ticket moves to `ready`.

---

### 1. Scaffold `apm-ui/`

Run from repo root (not committed; the output files are what gets committed):

```
npm create vite@latest apm-ui -- --template react-ts
cd apm-ui
npm install
npm install @tanstack/react-query
```

Then initialise shadcn/ui (base setup only — no components):

```
npx shadcn@latest init
```

Accept the defaults (TypeScript, Tailwind, CSS variables). This writes:
- `components.json`
- `src/lib/utils.ts` (the `cn` helper)
- `src/index.css` (CSS variables + Tailwind directives)
- Updates `tailwind.config.ts` and `vite.config.ts`

Add to `apm-ui/.gitignore`:
```
node_modules/
dist/
```

---

### 2. Wire TanStack Query — `apm-ui/src/main.tsx`

Wrap `<App />` with `QueryClientProvider`:

```tsx
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

const queryClient = new QueryClient()

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </React.StrictMode>,
)
```

---

### 3. Fetch tickets — `apm-ui/src/App.tsx`

Replace the default template content with a minimal component:

```tsx
import { useQuery } from '@tanstack/react-query'

function App() {
  const { data, error } = useQuery({
    queryKey: ['tickets'],
    queryFn: () => fetch('/api/tickets').then(r => {
      if (!r.ok) throw new Error(`/api/tickets returned ${r.status}`)
      return r.json()
    }),
  })

  if (data) console.log('tickets', data)
  if (error) console.error('tickets error', error)

  return <></>
}

export default App
```

The page renders nothing. All feedback is in the console.

---

### 4. Static file serving — `apm-server/src/main.rs`

`tower-http` was listed as an optional dependency in the Step 1 spec. Add the `ServeDir` feature:

In `apm-server/Cargo.toml`:
```toml
tower-http = { workspace = true, features = ["fs"] }
```

In root `Cargo.toml` workspace.dependencies, update or add:
```toml
tower-http = { version = "0.5", features = [] }
```

In `apm-server/src/main.rs`, add the static file route after existing routes:

```rust
use tower_http::services::{ServeDir, ServeFile};

let serve_dir = ServeDir::new("apm-ui/dist")
    .not_found_service(ServeFile::new("apm-ui/dist/index.html"));

let app = Router::new()
    .route("/health", get(health_handler))
    .route("/api/tickets", get(tickets_handler))
    .route("/api/tickets/:id", get(ticket_by_id_handler))
    .nest_service("/", serve_dir)
    .with_state(state);
```

`ServeFile` fallback ensures React Router (if added later) works; it's harmless here.

The server must be run from the **repo root** so the relative path `apm-ui/dist` resolves correctly. Document this in a comment above the `ServeDir::new` call.

---

### 5. Build and validate

Steps to confirm it works end-to-end:

1. `cd apm-ui && npm run build && cd ..`
2. `cargo run -p apm-server` (from repo root)
3. Open `http://localhost:3000/` — blank page, no errors
4. Browser console shows the JSON array from `/api/tickets`

---

### File changes summary

| File | Change |
|------|--------|
| `apm-ui/` | New directory — full Vite scaffold + shadcn/ui init |
| `apm-ui/.gitignore` | `node_modules/`, `dist/` |
| `apm-server/Cargo.toml` | Add `tower-http` with `fs` feature |
| `Cargo.toml` (root) | Add/update `tower-http` in workspace deps |
| `apm-server/src/main.rs` | Add `ServeDir` fallback route for `/` |

`node_modules/` and `dist/` are excluded from git; only source files are committed.

### Open questions



### Amendment requests



## History

| When | From | To | By |
|------|------|----|----|
| 2026-03-31T06:11Z | — | new | apm |
| 2026-03-31T06:16Z | new | in_design | philippepascal |
| 2026-03-31T06:19Z | in_design | specd | claude-0330-0800-f4a2 |
| 2026-03-31T19:43Z | specd | ready | apm |
| 2026-03-31T21:45Z | ready | in_progress | philippepascal |
| 2026-03-31T21:52Z | in_progress | implemented | claude-0331-2145-b7f2 |
| 2026-03-31T23:12Z | implemented | accepted | philippepascal |
| 2026-04-01T04:57Z | accepted | closed | apm-sync |
| 2026-04-06T06:49Z | closed | closed | apm |
