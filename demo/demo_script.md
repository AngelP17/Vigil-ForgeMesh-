# Vigil Demo Script

Target runtime: 5-6 minutes.

1. Frame the problem.
   Shift supervisors are buried in siloed machine logs, maintenance tickets, and operator notes. Raw anomalies are not enough; operations teams need explainable incidents and trusted next actions.

2. Seed noisy local data.
   Run `cargo run -p vigil-cli -- seed-demo` and point out the exported files in `data/`. Call out nulls, duplicates, delayed events, and conflicting notes.

3. Run the incident loop.
   Run `cargo run -p vigil-cli -- detect` and show that incidents are created from the synthetic but messy operational context.

4. Open the dashboard.
   Start `cargo run -p vigil-cli -- daemon --port 8080`, then open `http://localhost:8080/dashboard` (the landing page is at `/`; the incident list lives on `/dashboard`).

5. Walk the incident list.
   Open a temperature spike or vibration anomaly. Show the suspected cause, recommended action, linked maintenance context, and operator action controls.

6. Take an action.
   Record `assign_maintenance` or `resolve` with a note. Show the incident status update in the UI.

7. Open replay and verify integrity.
   Show the timeline, rules fired, Merkle root, proof array, and the `Valid Merkle path - data untampered` verification result.

8. Close with the positioning statement.
   Vigil converts noisy manufacturing signals into explainable incidents, operator decisions, and replayable audit trails.
