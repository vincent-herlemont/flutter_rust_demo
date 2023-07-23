# How run stack and dependencies overview

- `./supabase` depend of nothing.
- `./monitoring` depend of `./supabase`.
- `./services` depend of `./supabase`.

Advice run:
1. `./supabase/run.sh`
2. `./monitoring/run.sh`
3. `./services/run.sh`
