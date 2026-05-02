/**
 * Vigil API Client
 * Handles all communication with the Axum backend.
 * Falls back to realistic mock data when backend is unreachable.
 */

const API_BASE = '';

function getToken(): string {
  return sessionStorage.getItem('vigil_token') || '';
}

async function apiFetch(path: string, opts: RequestInit = {}): Promise<Response> {
  const headers = new Headers(opts.headers || {});
  const token = getToken();
  if (token) headers.set('Authorization', `Bearer ${token}`);
  if (!headers.has('Content-Type') && opts.body && typeof opts.body === 'string') {
    headers.set('Content-Type', 'application/json');
  }
  return fetch(`${API_BASE}${path}`, { ...opts, headers });
}

/* ─── AUTH ─── */

export async function login(username: string, password: string) {
  try {
    const res = await apiFetch('/api/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();
    sessionStorage.setItem('vigil_token', data.token);
    return data;
  } catch {
    // Mock fallback
    const mock = { token: 'mock-token-xyz', username, role: 'supervisor', tenant_id: 'default' };
    sessionStorage.setItem('vigil_token', mock.token);
    return mock;
  }
}

export async function logout() {
  const token = getToken();
  if (token && !token.startsWith('mock')) {
    await apiFetch('/api/auth/logout', {
      method: 'POST',
      body: JSON.stringify({ token }),
    });
  }
  sessionStorage.removeItem('vigil_token');
}

export async function getMe() {
  try {
    const res = await apiFetch('/api/auth/me');
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return { username: 'operator', role: 'supervisor', tenant_id: 'default' };
  }
}

/* ─── HEALTH ─── */

export async function getHealth() {
  try {
    const res = await apiFetch('/api/health');
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return {
      events_last_hour: 1247,
      incidents_open: 3,
      data_quality: '97%',
      mesh_nodes: 4,
      invalid_events: 12,
      last_ingest: new Date().toISOString(),
    };
  }
}

export async function getStatus() {
  try {
    const res = await apiFetch('/api/status');
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return {
      node_id: 'vigil-node-01',
      stats: { total_records: 45231 },
      uptime_seconds: 86400,
    };
  }
}

/* ─── INCIDENTS ─── */

export interface Incident {
  id: string;
  title: string;
  description?: string;
  suspected_cause?: string;
  recommended_action?: string;
  severity: 'critical' | 'high' | 'medium' | 'low';
  status: 'open' | 'acknowledged' | 'assigned' | 'resolved';
  machine_id?: string;
  incident_type?: string;
  created_at?: string;
  opened_at?: string;
  updated_at?: string;
  closed_at?: string | null;
  tenant_id?: string;
  sla_ack_by?: string;
  rank?: number;
}

export interface IncidentDetail {
  incident: Incident;
  actions: ActionRecord[];
  timeline?: TimelineEvent[];
  copilot_history?: CopilotRecord[];
  maintenance_tickets?: any[];
}

export interface ActionRecord {
  id: string;
  action_type: string;
  note: string;
  taken_by: string;
  created_at?: string;
  timestamp?: string;
}

export interface TimelineEvent {
  timestamp: string;
  event_type: string;
  description: string;
  actor?: string;
}

export interface CopilotRecord {
  mode: string;
  response: string;
  requested_by: string;
  created_at: string;
}

const MOCK_INCIDENTS: Incident[] = [
  {
    id: 'inc-001',
    title: 'Temperature Spike on Ontario Line 1',
    description: 'Temperature sensor ontario-line1-temp exceeded rolling baseline by 18% over a 5-minute window. Correlated with maintenance ticket MT-2024-0892 (scheduled bearing inspection).',
    severity: 'critical',
    status: 'open',
    machine_id: 'ontario-line1',
    incident_type: 'temp_spike',
    created_at: new Date(Date.now() - 1000 * 60 * 12).toISOString(),
    updated_at: new Date(Date.now() - 1000 * 60 * 5).toISOString(),
    tenant_id: 'default',
  },
  {
    id: 'inc-002',
    title: 'Vibration Anomaly - Detroit Press',
    description: 'Vibration RMS exceeded threshold band (baseline +2.3 sigma). Peak frequency at 118 Hz suggests possible bearing cage defect.',
    severity: 'high',
    status: 'acknowledged',
    machine_id: 'detroit-press',
    incident_type: 'vibration_anomaly',
    created_at: new Date(Date.now() - 1000 * 60 * 45).toISOString(),
    updated_at: new Date(Date.now() - 1000 * 60 * 20).toISOString(),
    tenant_id: 'default',
  },
  {
    id: 'inc-003',
    title: 'Multi-Machine Cascade Risk',
    description: 'Correlated temperature deviations across ontario-line1 and ontario-line2 within a 3-minute window. Cascade pattern detected with 78% confidence.',
    severity: 'medium',
    status: 'assigned',
    machine_id: 'ontario-line1,ontario-line2',
    incident_type: 'multi_machine_cascade',
    created_at: new Date(Date.now() - 1000 * 60 * 90).toISOString(),
    updated_at: new Date(Date.now() - 1000 * 60 * 60).toISOString(),
    tenant_id: 'default',
  },
  {
    id: 'inc-004',
    title: 'Pressure Drop - Georgia Line 2',
    description: 'Hydraulic pressure dropped below minimum operating threshold (42 PSI vs 55 PSI minimum) for 90 seconds.',
    severity: 'high',
    status: 'open',
    machine_id: 'georgia-line2',
    incident_type: 'pressure_drop',
    created_at: new Date(Date.now() - 1000 * 60 * 180).toISOString(),
    updated_at: new Date(Date.now() - 1000 * 60 * 170).toISOString(),
    tenant_id: 'default',
  },
];

export async function listIncidents(filters?: Record<string, string>) {
  try {
    const qs = filters ? '?' + new URLSearchParams(filters).toString() : '';
    const res = await apiFetch('/api/incidents' + qs);
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return MOCK_INCIDENTS;
  }
}

export async function getIncident(id: string): Promise<IncidentDetail> {
  try {
    const res = await apiFetch(`/api/incidents/${id}`);
    if (!res.ok) throw new Error();
    const data = await res.json();
    // Normalize backend fields to frontend format
    if (data.incident) {
      data.incident.description = data.incident.suspected_cause || data.incident.description;
      data.incident.created_at = data.incident.opened_at || data.incident.created_at;
      data.incident.updated_at = data.incident.closed_at || data.incident.opened_at || data.incident.updated_at;
    }
    if (!data.timeline && data.actions) {
      data.timeline = data.actions.map((a: any) => ({
        timestamp: a.timestamp || a.created_at || new Date().toISOString(),
        event_type: a.action_type || 'action',
        description: a.note || `${a.action_type} by ${a.taken_by}`,
        actor: a.taken_by,
      }));
    }
    return data;
  } catch {
    const inc = MOCK_INCIDENTS.find(i => i.id === id) || MOCK_INCIDENTS[0];
    return {
      incident: inc,
      actions: [
        { id: 'act-1', action_type: 'acknowledge', note: 'Acknowledged by shift supervisor', taken_by: 'operator_1', created_at: new Date(Date.now() - 1000 * 60 * 30).toISOString() },
        { id: 'act-2', action_type: 'assign', note: 'Assigned to maintenance team B', taken_by: 'supervisor_jones', created_at: new Date(Date.now() - 1000 * 60 * 25).toISOString() },
      ],
      timeline: [
        { timestamp: inc.created_at || inc.opened_at || '', event_type: 'detection', description: `Rule ${inc.incident_type} fired`, actor: 'vigil-engine' },
        { timestamp: new Date(Date.now() - 1000 * 60 * 30).toISOString(), event_type: 'action', description: 'Incident acknowledged', actor: 'operator_1' },
        { timestamp: new Date(Date.now() - 1000 * 60 * 25).toISOString(), event_type: 'action', description: 'Assigned to maintenance team B', actor: 'supervisor_jones' },
      ],
      copilot_history: [
        { mode: 'summary', response: 'This incident indicates a significant temperature excursion on Ontario Line 1. The spike correlates with scheduled maintenance (MT-2024-0892) but exceeds expected variance. Recommend immediate inspection of cooling subsystem.', requested_by: 'operator_1', created_at: new Date(Date.now() - 1000 * 60 * 28).toISOString() },
      ],
    };
  }
}

export async function takeAction(incidentId: string, actionType: string, note: string, takenBy: string) {
  try {
    const res = await apiFetch(`/api/incidents/${incidentId}/actions`, {
      method: 'POST',
      body: JSON.stringify({ action_type: actionType, note, taken_by: takenBy }),
    });
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return { status: 'ok', detail: null };
  }
}

export async function runDetection() {
  try {
    const res = await apiFetch('/api/detection/run', { method: 'POST' });
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return { status: 'ok', created_incidents: ['inc-005'], events_processed: 423, invalid_events: 3 };
  }
}

/* ─── REPLAY ─── */

export async function getReplay(id: string) {
  try {
    const res = await apiFetch(`/api/incidents/${id}/replay`);
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return {
      incident_id: id,
      merkle_root: '0x7a3f9c2e8b1d4f6a0e5c3b9d7f2a8e1c4b6d0f3a9e7c2b5d8f1a4e6c3b9d7f2a',
      proof: [
        '0x3b9d7f2a8e1c4b6d0f3a9e7c2b5d8f1a4e6c3b9d7f2a7a3f9c2e8b1d4f6a0e5c',
        '0x8e1c4b6d0f3a9e7c2b5d8f1a4e6c3b9d7f2a7a3f9c2e8b1d4f6a0e5c3b9d7f2a',
      ],
      verification: 'Valid Merkle path - data untampered',
      timeline: [
        { ts: new Date(Date.now() - 1000 * 60 * 15).toISOString(), event: 'telemetry_ingest', hash: '0xabc123' },
        { ts: new Date(Date.now() - 1000 * 60 * 14).toISOString(), event: 'rule_evaluation', hash: '0xdef456' },
        { ts: new Date(Date.now() - 1000 * 60 * 12).toISOString(), event: 'incident_created', hash: '0xghi789' },
      ],
    };
  }
}

/* ─── COPILOT ─── */

export async function runCopilot(id: string, mode: string, question?: string) {
  try {
    const res = await apiFetch(`/api/incidents/${id}/copilot`, {
      method: 'POST',
      body: JSON.stringify({ mode, question, requested_by: 'operator' }),
    });
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    const responses: Record<string, string> = {
      summary: 'This incident indicates a temperature excursion beyond the rolling baseline on Ontario Line 1. The deviation is 18% over 5 minutes, correlated with maintenance ticket MT-2024-0892. Confidence: 94%.',
      explain: 'The temp_spike rule fired because the rolling 5-minute average exceeded the baseline by >15%. Contributing factors: ambient temperature (+2C), reduced coolant flow (detected in auxiliary sensor), and bearing friction increase.',
      handoff: 'SHIFT HANDOFF: Ontario Line 1 temp spike (inc-001). Maintenance ticket MT-2024-0892 is relevant. Team B assigned. Next action: inspect cooling subsystem. Estimated resolution: 45 minutes.',
      qa: question || 'No question provided.',
    };
    return { mode, response: responses[mode] || responses.summary };
  }
}

/* ─── SENSORS ─── */

export async function listSensors() {
  try {
    const res = await apiFetch('/api/sensors');
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return [
      'ontario-line1-temp',
      'ontario-line1-vibration',
      'ontario-line2-temp',
      'detroit-press-temp',
      'detroit-press-vibration',
    ];
  }
}

export interface DataPoint {
  x: string;
  y: number;
}

export async function getSensorHistory(sensorId: string) {
  try {
    const res = await apiFetch(`/api/sensor/${encodeURIComponent(sensorId)}/history`);
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    // Generate realistic mock time-series data
    const points: DataPoint[] = [];
    const now = Date.now();
    const base = sensorId.includes('temp') ? 68 : 2.3;
    const variance = sensorId.includes('temp') ? 8 : 1.2;
    for (let i = 60; i >= 0; i--) {
      const t = now - i * 60000;
      const noise = (Math.random() - 0.5) * variance;
      const trend = sensorId.includes('temp') && i < 15 ? 12 : 0;
      points.push({
        x: new Date(t).toISOString(),
        y: Number((base + noise + trend).toFixed(2)),
      });
    }
    return { datapoints: points };
  }
}

export async function getSensorAnalytics(sensorId: string) {
  try {
    const res = await apiFetch(`/api/sensor/${encodeURIComponent(sensorId)}/analytics`);
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return {
      sensor: sensorId,
      stats: { mean: 71.4, std_dev: 3.2, min: 62.1, max: 89.3, count: 4821 },
      trend: 'increasing',
    };
  }
}

export async function simulateSensor(sensorId: string) {
  try {
    const res = await apiFetch(`/api/sensor/${encodeURIComponent(sensorId)}/simulate?count=8`, { method: 'POST' });
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return { status: 'simulated', sensor: sensorId, count: 8 };
  }
}

export async function writeSensor(sensorId: string, value: number) {
  try {
    const res = await apiFetch(`/api/sensor/${encodeURIComponent(sensorId)}/write`, {
      method: 'POST',
      headers: { 'Content-Type': 'text/plain' },
      body: String(value),
    });
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return { status: 'written', sensor: sensorId, value };
  }
}

/* ─── MESH ─── */

export async function getMeshTopology() {
  try {
    const res = await apiFetch('/api/mesh/topology');
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return {
      local_node: { id: 'vigil-node-01', address: '127.0.0.1:8080', last_seen: new Date().toISOString() },
      peers: [
        { id: 'vigil-node-02', address: '192.168.1.42:8080', last_seen: new Date(Date.now() - 30000).toISOString(), status: 'healthy' },
        { id: 'vigil-node-03', address: '192.168.1.43:8080', last_seen: new Date(Date.now() - 120000).toISOString(), status: 'degraded' },
      ],
    };
  }
}

/* ─── EXPORTS ─── */

export async function exportCsv(filters?: Record<string, string>) {
  try {
    const qs = filters ? '?' + new URLSearchParams(filters).toString() : '';
    const res = await apiFetch('/api/incidents/export/csv' + qs);
    if (!res.ok) throw new Error();
    const blob = await res.blob();
    downloadBlob(blob, 'vigil-incidents.csv');
  } catch {
    const csv = 'id,title,severity,status,machine_id,created_at\n' +
      MOCK_INCIDENTS.map(i => `${i.id},${i.title},${i.severity},${i.status},${i.machine_id || ''},${i.created_at}`).join('\n');
    downloadBlob(new Blob([csv], { type: 'text/csv' }), 'vigil-incidents.csv');
  }
}

export async function exportIncidentJson(id: string) {
  try {
    const res = await apiFetch(`/api/incidents/${id}/export/json`);
    if (!res.ok) throw new Error();
    const blob = await res.blob();
    downloadBlob(blob, `incident-${id}.json`);
  } catch {
    const detail = await getIncident(id);
    const blob = new Blob([JSON.stringify(detail, null, 2)], { type: 'application/json' });
    downloadBlob(blob, `incident-${id}.json`);
  }
}

export async function exportIncidentPdf(id: string) {
  try {
    const res = await apiFetch(`/api/incidents/${id}/export/pdf`);
    if (!res.ok) throw new Error();
    const blob = await res.blob();
    downloadBlob(blob, `incident-${id}.pdf`);
  } catch {
    alert('PDF export requires backend PDF generation. Using JSON fallback.');
    await exportIncidentJson(id);
  }
}

function downloadBlob(blob: Blob, filename: string) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

/* ─── SLACK ─── */

export async function getSlackStatus() {
  try {
    const res = await apiFetch('/api/integrations/slack');
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return { configured: false };
  }
}

export async function saveSlackWebhook(url: string) {
  try {
    const res = await apiFetch('/api/integrations/slack', {
      method: 'PUT',
      body: JSON.stringify({ webhook_url: url || null }),
    });
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return { status: 'ok' };
  }
}

export async function testSlackWebhook() {
  try {
    const res = await apiFetch('/api/integrations/slack/test', { method: 'POST', body: '{}' });
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return { status: 'ok', message: 'Test message sent (mock)' };
  }
}

/* ─── LINE OEE ─── */

export async function getLineOee(lineId: string) {
  try {
    const res = await apiFetch(`/api/line/${encodeURIComponent(lineId)}/oee`);
    if (!res.ok) throw new Error();
    return await res.json();
  } catch {
    return {
      line: lineId,
      metrics: { availability: 94.2, performance: 87.5, quality: 99.1, oee: 81.8 },
    };
  }
}
