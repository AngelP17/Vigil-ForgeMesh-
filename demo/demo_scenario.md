# Demo Scenario

The demo environment models three plants:

- `ontario-line1`: temperature drift and cooling fan degradation.
- `georgia-line2`: vibration escalation and bearing wear.
- `texas-line3`: cascade pressure from rerouted work and recurring conveyor instability.

Each plant emits:

- machine PLC telemetry
- maintenance tickets
- operator notes

Injected realism:

- null PLC values
- delayed and out-of-order arrivals
- duplicate events
- free-text notes that partially conflict with machine telemetry

Expected incidents:

- `temp_spike`
- `vibration_anomaly`
- `multi_machine_cascade`
