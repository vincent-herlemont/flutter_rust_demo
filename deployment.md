# Deployment

Goals:
- Coherence: All server system parts must be at the same versions.
- Client experiences: During a downtime, all clients need to know that is a maintenance.

---

API:
- Table deployments (public read access, service roles create/update/delete access):
  - versions number: semver unique
  - created_at: not null
  - status: ('scheduled', 'running', 'failed', 'completed', 'finished') 
  - updated_at: not null

Rules:
- Only 1 version can be set to "completed".
- When a version is schedule or running, all services stop to requesting to instead is update to date.
