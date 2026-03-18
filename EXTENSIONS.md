# Vigil Extension Notes

## Implemented in v1

### Operational workflow layer

- SQLite-backed incident storage
- operator actions with status transitions
- decision audit log with reasoning snapshots
- replay endpoint with Merkle verification output

### Demo realism

- machine PLC telemetry
- maintenance ticket context
- operator note context
- nulls, duplicates, delays, out-of-order arrivals, and conflicting observations

### UI

- incident list
- incident detail
- replay / audit pane
- health strip

## Intentionally not built in v1

- auth
- cloud deployment
- notifications
- rule editor
- heavyweight ML
- role system

## Next plausible extensions

- incident acknowledgement SLA tracking
- richer replay proof visualization
- site-to-site sync status tied into incident routing
- exportable incident packets for audit review
