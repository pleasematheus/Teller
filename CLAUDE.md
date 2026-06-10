 # Teller: Project Guidelines

## Approach
- Read files before writing. Do not re-read unless they have changed.
- Thorough reasoning, concise output.
- Skip files above 100KB unless necessary.
- No sycophantic openings or closing fluff.
- No emojis or em-dashes.
- Do not guess APIs, versions, flags, SHAs, or package names. Verify by reading code or docs before stating; if nothing works, ask.

## Forbidden
- Never rewrite an entire file to change one function; edit only the necessary snippet.
- Never add dependencies without asking first.
- Never create tests without first reading the file to be tested.
- Never read the full RTK catalog (docs/rtk.md) without explicit need.

## Functional Programming
- Prefer pure functions: same input, same output, no hidden side effects.
- Immutable data: never mutate objects or arrays; use spread, map, filter, reduce.
- Avoid classes; use functions and composition.
- React components are functions; side effects only in explicit hooks (useEffect, useCallback).
- Extract business logic from components into pure, testable functions in src/lib/.
- In Rust, prefer functions that return Result<T, E> over implicit mutations; avoid shared mutable state.

## Tests
- Every utility in src/lib/ has a corresponding *.test.ts (Vitest) under tests/lib/, importing the module via the `@/lib/` alias.
- Every Tauri Rust command has at least one unit test in the same file (#[cfg(test)]).
- Test behavior, not implementation.
- Use the #### Scenario: blocks from OpenSpec specs as test case references.
- Manual tests must be included as tasks and delegated to human testers, not automated.
- Changes to the DuckDB database must include tasks for deleting the old version and creating the new one.
- Every incompatible change to `schema.sql` requires (a) incrementing `DB_VERSION` in `db.rs` and (b) a new `src/db/migrations/NNN.sql` or Rust step appended to the `MIGRATIONS` list in `db.rs`. Never edit or remove released migration entries.
- Spawn a subagent to run tests and report results back to the main agent.

## Stack
- Frontend: React 19, TypeScript, Tailwind 4, shadcn/ui, Vitest
- Backend: Tauri 2, Rust, DuckDB (duckdb = "1.4.4" bundled)
- Tests: `cargo test` (Rust), `bun run test` (frontend)