import { useEffect, useRef, useState, useCallback } from 'react'
import { Link, useLocation } from 'react-router-dom'
import {
  Activity,
  AlertTriangle,
  BarChart3,
  BrainCircuit,
  CheckCircle2,
  ChevronLeft,
  Download,
  FileText,
  Filter,
  HeartPulse,
  LogIn,
  LogOut,
  Mail,
  Network,
  Play,
  RefreshCw,
  Send,
  Settings,
  ShieldCheck,
  Trash2,
  Zap,
} from 'lucide-react'
import Chart from 'chart.js/auto'
import type { Chart as ChartType } from 'chart.js'
import * as api from '../lib/api'
import type { Incident, IncidentDetail } from '../lib/api'

/* ─── THEME TOGGLE ─── */
function useTheme() {
  const [theme, setTheme] = useState<'dark' | 'light'>('dark')
  useEffect(() => {
    const stored = localStorage.getItem('vigil_theme') as 'dark' | 'light' | null
    const initial = stored || 'dark'
    setTheme(initial)
    document.documentElement.setAttribute('data-theme', initial)
  }, [])
  const toggle = () => {
    const next = theme === 'dark' ? 'light' : 'dark'
    setTheme(next)
    document.documentElement.setAttribute('data-theme', next)
    localStorage.setItem('vigil_theme', next)
  }
  return { theme, toggle }
}

/* ─── BADGE ─── */
function SeverityBadge({ severity }: { severity: string }) {
  const map: Record<string, string> = {
    critical: 'bg-red-500/15 text-red-400 border-red-500/30',
    high: 'bg-amber-500/15 text-amber-400 border-amber-500/30',
    medium: 'bg-[var(--vigil-accent)]/15 text-[var(--vigil-accent)] border-[var(--vigil-accent)]/30',
    low: 'bg-slate-500/15 text-slate-400 border-slate-500/30',
  }
  return (
    <span className={`inline-block px-3 py-1 rounded-full text-[11px] font-bold font-mono border uppercase tracking-wide ${map[severity] || map.low}`}>
      {severity}
    </span>
  )
}

function StatusBadge({ status }: { status: string }) {
  const map: Record<string, string> = {
    open: 'bg-[var(--vigil-accent)]/10 text-[var(--vigil-accent)] border-[var(--vigil-accent)]/20',
    acknowledged: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
    assigned: 'bg-purple-500/10 text-purple-400 border-purple-500/20',
    resolved: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
  }
  return (
    <span className={`inline-block px-3 py-1 rounded-full text-[11px] font-bold font-mono border uppercase tracking-wide ${map[status] || map.open}`}>
      {status}
    </span>
  )
}

/* ─── SIDEBAR ─── */
function Sidebar({ active, counts }: { active: string; counts: { incidents: number; open: number } }) {
  const navItems = [
    { id: 'incidents', label: 'Incidents', icon: <AlertTriangle size={18} />, badge: counts.open },
    { id: 'health', label: 'System Health', icon: <HeartPulse size={18} /> },
    { id: 'telemetry', label: 'Sensor Trends', icon: <BarChart3 size={18} /> },
    { id: 'mesh', label: 'Mesh Topology', icon: <Network size={18} /> },
  ]

  return (
    <aside className="w-64 bg-[var(--vigil-bg2)] border-r border-[var(--vigil-border)] flex flex-col h-full">
      <div className="p-5 border-b border-[var(--vigil-border)]">
        <Link to="/" className="flex items-center gap-2.5 text-[var(--vigil-text)] font-bold text-lg tracking-tight no-underline">
          <svg viewBox="0 0 28 28" width={22} height={22} fill="none">
            <path d="M14 2L2 26L14 20L26 26L14 2Z" fill="#f59e0b" />
            <path d="M14 2L14 20L2 26L14 2Z" fill="#d97706" />
          </svg>
          Vigil
        </Link>
      </div>
      <div className="p-4 flex-1 overflow-y-auto">
        <div className="text-[10px] font-bold uppercase tracking-[0.15em] text-[var(--vigil-dim)] mb-3 px-3">Navigation</div>
        {navItems.map(item => (
          <a
            key={item.id}
            href={`#${item.id}`}
            onClick={(e) => { e.preventDefault(); window.location.hash = item.id }}
            className={`flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm mb-1 transition-all cursor-pointer ${
              active === item.id
                ? 'bg-[var(--vigil-accent)]/10 text-[var(--vigil-accent)] border border-[var(--vigil-accent)]/20'
                : 'text-[var(--vigil-muted)] hover:bg-white/[0.04] hover:text-[var(--vigil-text)]'
            }`}
          >
            {item.icon}
            <span className="flex-1">{item.label}</span>
            {item.badge !== undefined && item.badge > 0 && (
              <span className="px-2 py-0.5 rounded-full text-[10px] font-bold bg-red-500/15 text-red-400 font-mono">{item.badge}</span>
            )}
          </a>
        ))}
      </div>
    </aside>
  )
}

/* ─── LOGIN BAR ─── */
function LoginBar({ onRefresh }: { onRefresh: () => void }) {
  const [user, setUser] = useState('')
  const [pass, setPass] = useState('')
  const [me, setMe] = useState<{ username?: string; role?: string } | null>(null)

  const refreshMe = useCallback(async () => {
    try {
      const m = await api.getMe()
      setMe(m)
    } catch {
      setMe(null)
    }
  }, [])

  useEffect(() => { refreshMe() }, [refreshMe])

  async function doLogin() {
    await api.login(user || 'operator', pass || 'vigil')
    setUser(''); setPass('')
    await refreshMe()
    onRefresh()
  }

  async function doLogout() {
    await api.logout()
    setMe(null)
    onRefresh()
  }

  return (
    <div className="flex items-center gap-3 flex-wrap">
      {!me?.username ? (
        <>
          <input
            value={user}
            onChange={e => setUser(e.target.value)}
            placeholder="user"
            className="px-3 py-1.5 rounded-lg bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)] w-28"
          />
          <input
            type="password"
            value={pass}
            onChange={e => setPass(e.target.value)}
            placeholder="password"
            className="px-3 py-1.5 rounded-lg bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)] w-28"
          />
          <button onClick={doLogin} className="px-4 py-1.5 rounded-lg text-sm font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all flex items-center gap-1.5">
            <LogIn size={14} /> Sign in
          </button>
        </>
      ) : (
        <span className="text-xs text-[var(--vigil-muted)] font-mono">{me.username} · {me.role}</span>
      )}
      {me?.username && (
        <button onClick={doLogout} className="px-4 py-1.5 rounded-lg text-sm font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-red-500/50 hover:text-red-400 transition-all flex items-center gap-1.5">
          <LogOut size={14} /> Sign out
        </button>
      )}
    </div>
  )
}

/* ─── HEALTH STRIP ─── */
function HealthStrip({ health }: { health: any }) {
  const items = [
    { label: 'Events/Hour', value: health.events_last_hour ?? '—' },
    { label: 'Open Incidents', value: health.incidents_open ?? '—' },
    { label: 'Data Quality', value: health.data_quality ?? '—' },
    { label: 'Mesh Nodes', value: health.mesh_nodes ?? '—' },
    { label: 'Invalid Events', value: health.invalid_events ?? '—' },
  ]
  return (
    <div className="grid grid-cols-2 md:grid-cols-5 gap-4 p-5 rounded-2xl border border-[var(--vigil-border)] mb-6" style={{ background: 'linear-gradient(160deg, var(--vigil-card), var(--vigil-bg2))' }}>
      {items.map((item, i) => (
        <div key={i} className="text-center">
          <div className="text-2xl font-extrabold font-mono tracking-tight text-[var(--vigil-text)]">{item.value}</div>
          <div className="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--vigil-muted)] mt-1">{item.label}</div>
        </div>
      ))}
    </div>
  )
}

/* ─── INCIDENT LIST VIEW ─── */
function IncidentListView({ onSelect }: { onSelect: (id: string) => void }) {
  const [incidents, setIncidents] = useState<Incident[]>([])
  const [loading, setLoading] = useState(true)
  const [filters, setFilters] = useState({ severity: '', machine: '', q: '', from: '', to: '' })

  async function load() {
    setLoading(true)
    const data = await api.listIncidents()
    setIncidents(data)
    setLoading(false)
  }

  useEffect(() => { load() }, [])

  async function applyFilters() {
    setLoading(true)
    const f: Record<string, string> = {}
    if (filters.severity) f.severity = filters.severity
    if (filters.machine) f.machine = filters.machine
    if (filters.q) f.q = filters.q
    if (filters.from) f.from = filters.from
    if (filters.to) f.to = filters.to
    const data = await api.listIncidents(Object.keys(f).length ? f : undefined)
    setIncidents(data)
    setLoading(false)
  }

  function clearFilters() {
    setFilters({ severity: '', machine: '', q: '', from: '', to: '' })
    load()
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center py-20 text-[var(--vigil-muted)] text-sm">
        <RefreshCw size={16} className="animate-spin mr-2" /> Loading incidents...
      </div>
    )
  }

  return (
    <div>
      <div className="mb-6">
        <div className="text-xs font-mono text-[var(--vigil-dim)] mb-1">vigil:// / <span className="text-[var(--vigil-accent)]">incidents</span></div>
        <h1 className="text-3xl font-extrabold tracking-tight text-[var(--vigil-text)] mb-1">Operational Incidents</h1>
        <p className="text-sm text-[var(--vigil-muted)]">Explainable incidents detected from machine logs, maintenance tickets, and operator notes</p>
      </div>

      <HealthStrip health={{ events_last_hour: 1247, incidents_open: incidents.filter(i => i.status === 'open').length, data_quality: '97%', mesh_nodes: 4, invalid_events: 12 }} />

      {/* Filters */}
      <div className="p-4 rounded-2xl border border-[var(--vigil-border)] mb-6 bg-[var(--vigil-card)]/40">
        <div className="text-sm font-bold text-[var(--vigil-text)] mb-3 flex items-center gap-2"><Filter size={14} /> Filters</div>
        <div className="flex flex-wrap gap-3">
          <input value={filters.severity} onChange={e => setFilters({ ...filters, severity: e.target.value })} placeholder="Severity" className="px-3 py-2 rounded-xl bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)] w-32" />
          <input value={filters.machine} onChange={e => setFilters({ ...filters, machine: e.target.value })} placeholder="Machine / line" className="px-3 py-2 rounded-xl bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)] w-40" />
          <input value={filters.q} onChange={e => setFilters({ ...filters, q: e.target.value })} placeholder="Search title / id" className="px-3 py-2 rounded-xl bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)] min-w-[200px]" />
          <input value={filters.from} onChange={e => setFilters({ ...filters, from: e.target.value })} placeholder="From (RFC3339)" className="px-3 py-2 rounded-xl bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)] w-40" />
          <input value={filters.to} onChange={e => setFilters({ ...filters, to: e.target.value })} placeholder="To (RFC3339)" className="px-3 py-2 rounded-xl bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)] w-40" />
          <button onClick={applyFilters} className="px-4 py-2 rounded-xl text-sm font-bold bg-[var(--vigil-accent)] text-slate-950 hover:brightness-110 transition-all">Apply</button>
          <button onClick={clearFilters} className="px-4 py-2 rounded-xl text-sm font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all">Clear</button>
          <button onClick={() => api.exportCsv()} className="px-4 py-2 rounded-xl text-sm font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all flex items-center gap-1.5">
            <Download size={14} /> CSV
          </button>
        </div>
      </div>

      {/* Incident cards */}
      <div className="flex flex-col gap-4">
        {incidents.length === 0 && (
          <div className="text-center py-16 text-[var(--vigil-muted)]">
            <AlertTriangle size={48} className="mx-auto mb-4 opacity-30" />
            <h3 className="text-lg font-bold text-[var(--vigil-text)] mb-1">No incidents found</h3>
            <p className="text-sm">Try clearing filters or run detection to generate incidents.</p>
          </div>
        )}
        {incidents.map(inc => (
          <div
            key={inc.id}
            data-id={inc.id}
            onClick={() => onSelect(inc.id)}
            className="group p-5 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)] cursor-pointer hover:border-[var(--vigil-accent)]/30 hover:translate-y-[-2px] transition-all duration-300"
          >
            <div className="flex items-start justify-between mb-3">
              <h3 className="text-lg font-bold text-[var(--vigil-text)] group-hover:text-[var(--vigil-accent)] transition-colors">{inc.title}</h3>
              <div className="flex gap-2">
                <SeverityBadge severity={inc.severity} />
                <StatusBadge status={inc.status} />
              </div>
            </div>
              <p className="text-sm text-[var(--vigil-muted)] leading-relaxed mb-4 line-clamp-2">
                {inc.suspected_cause || inc.description || inc.recommended_action || 'No description available'}
              </p>
              <div className="flex items-center justify-between pt-3 border-t border-[var(--vigil-border)]">
                <span className="text-xs font-mono text-[var(--vigil-dim)]">{inc.machine_id || '—'}</span>
                <span className="text-xs text-[var(--vigil-dim)]">
                  {inc.opened_at || inc.created_at ? new Date(inc.opened_at || inc.created_at || '').toLocaleString() : '—'}
                </span>
              </div>
          </div>
        ))}
      </div>
    </div>
  )
}

/* ─── INCIDENT DETAIL VIEW ─── */
function IncidentDetailView({ id, onBack }: { id: string; onBack: () => void }) {
  const [detail, setDetail] = useState<IncidentDetail | null>(null)
  const [replay, setReplay] = useState<any>(null)
  const [copilotMode, setCopilotMode] = useState('summary')
  const [copilotResponse, setCopilotResponse] = useState('')
  const [copilotLoading, setCopilotLoading] = useState(false)
  const [actionNote, setActionNote] = useState('')
  const [actionLoading, setActionLoading] = useState(false)

  async function load() {
    const d = await api.getIncident(id)
    setDetail(d)
    const r = await api.getReplay(id)
    setReplay(r)
  }

  useEffect(() => { load() }, [id])

  async function runCopilot() {
    setCopilotLoading(true)
    const res = await api.runCopilot(id, copilotMode)
    setCopilotResponse(res.response)
    setCopilotLoading(false)
  }

  async function takeAction(actionType: string) {
    setActionLoading(true)
    await api.takeAction(id, actionType, actionNote, 'operator')
    setActionNote('')
    await load()
    setActionLoading(false)
  }

  if (!detail) {
    return (
      <div className="flex items-center justify-center py-20 text-[var(--vigil-muted)] text-sm">
        <RefreshCw size={16} className="animate-spin mr-2" /> Loading incident...
      </div>
    )
  }

  const inc = detail.incident

  return (
    <div>
      <button onClick={onBack} className="mb-4 px-4 py-2 rounded-xl text-sm font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all flex items-center gap-1.5">
        <ChevronLeft size={16} /> Back to Incidents
      </button>

      <div className="text-xs font-mono text-[var(--vigil-dim)] mb-1">vigil:// / incidents / <span className="text-[var(--vigil-accent)]">{inc.id}</span></div>
      <h1 className="text-2xl md:text-3xl font-extrabold tracking-tight text-[var(--vigil-text)] mb-6">{inc.title}</h1>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Main column */}
        <div className="lg:col-span-2 flex flex-col gap-6">
          {/* Overview */}
          <div className="p-6 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
            <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-4 flex items-center gap-2"><FileText size={16} className="text-[var(--vigil-accent)]" /> Overview</h3>
            <div className="grid grid-cols-2 md:grid-cols-3 gap-4 mb-4">
              <div><div className="text-[10px] font-bold uppercase tracking-wider text-[var(--vigil-dim)] mb-1">Severity</div><SeverityBadge severity={inc.severity} /></div>
              <div><div className="text-[10px] font-bold uppercase tracking-wider text-[var(--vigil-dim)] mb-1">Status</div><StatusBadge status={inc.status} /></div>
              <div><div className="text-[10px] font-bold uppercase tracking-wider text-[var(--vigil-dim)] mb-1">Type</div><div className="text-sm text-[var(--vigil-text)] font-mono">{inc.incident_type || '—'}</div></div>
              <div><div className="text-[10px] font-bold uppercase tracking-wider text-[var(--vigil-dim)] mb-1">Machine</div><div className="text-sm text-[var(--vigil-text)] font-mono">{inc.machine_id || '—'}</div></div>
              <div><div className="text-[10px] font-bold uppercase tracking-wider text-[var(--vigil-dim)] mb-1">Opened</div><div className="text-sm text-[var(--vigil-muted)]">{inc.opened_at || inc.created_at ? new Date(inc.opened_at || inc.created_at || '').toLocaleString() : '—'}</div></div>
              <div><div className="text-[10px] font-bold uppercase tracking-wider text-[var(--vigil-dim)] mb-1">SLA Ack By</div><div className="text-sm text-[var(--vigil-muted)]">{inc.sla_ack_by ? new Date(inc.sla_ack_by).toLocaleString() : '—'}</div></div>
            </div>
            <div className="p-4 rounded-xl bg-black/20 border border-white/5 mb-4">
              <div className="text-[10px] font-bold uppercase tracking-wider text-[var(--vigil-dim)] mb-2">Suspected Cause</div>
              <p className="text-sm text-[var(--vigil-muted)] leading-relaxed">{inc.suspected_cause || inc.description || '—'}</p>
            </div>
            {inc.recommended_action && (
              <div className="p-4 rounded-xl bg-[var(--vigil-accent)]/5 border border-[var(--vigil-accent)]/10">
                <div className="text-[10px] font-bold uppercase tracking-wider text-[var(--vigil-accent)] mb-2">Recommended Action</div>
                <p className="text-sm text-[var(--vigil-text)] leading-relaxed">{inc.recommended_action}</p>
              </div>
            )}
          </div>

          {/* Timeline */}
          <div className="p-6 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
            <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-4 flex items-center gap-2"><RefreshCw size={16} className="text-[var(--vigil-accent)]" /> Action Timeline</h3>
            <div className="flex flex-col gap-3">
              {(!detail.timeline || detail.timeline.length === 0) && <p className="text-sm text-[var(--vigil-muted)]">No timeline events yet.</p>}
              {detail.timeline?.map((evt, i) => (
                <div key={i} className="flex gap-3 p-3 rounded-xl bg-white/[0.02] border border-white/5">
                  <div className="w-9 h-9 rounded-lg flex items-center justify-center bg-[var(--vigil-accent)]/10 border border-[var(--vigil-accent)]/20 text-[var(--vigil-accent)] flex-shrink-0">
                    <Activity size={16} />
                  </div>
                  <div className="flex-1 min-w-0">
                    <div className="text-sm font-semibold text-[var(--vigil-text)]">{evt.event_type}</div>
                    <div className="text-xs text-[var(--vigil-muted)]">{evt.description}</div>
                    <div className="text-[10px] font-mono text-[var(--vigil-dim)] mt-1">{evt.actor || 'system'} · {evt.timestamp ? new Date(evt.timestamp).toLocaleString() : '—'}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Maintenance Tickets */}
          {detail.maintenance_tickets && detail.maintenance_tickets.length > 0 && (
            <div className="p-6 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
              <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-4 flex items-center gap-2"><FileText size={16} className="text-[var(--vigil-accent)]" /> Maintenance Tickets</h3>
              <div className="flex flex-col gap-3">
                {detail.maintenance_tickets.map((ticket: any, i: number) => (
                  <div key={i} className="p-3 rounded-xl bg-white/[0.02] border border-white/5">
                    <div className="text-sm font-semibold text-[var(--vigil-text)]">{ticket.ticket_type || 'Ticket'} · {ticket.machine_id}</div>
                    <div className="text-xs text-[var(--vigil-muted)] mt-1">{ticket.description}</div>
                    <div className="text-[10px] font-mono text-[var(--vigil-dim)] mt-1">{ticket.status} · {ticket.opened_at ? new Date(ticket.opened_at).toLocaleString() : '—'}</div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Copilot */}
          <div className="p-6 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
            <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-4 flex items-center gap-2"><BrainCircuit size={16} className="text-[var(--vigil-accent)]" /> Copilot</h3>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-2 mb-4">
              {['summary', 'explain', 'handoff', 'qa'].map(mode => (
                <button
                  key={mode}
                  onClick={() => setCopilotMode(mode)}
                  className={`px-3 py-2 rounded-xl text-xs font-semibold border transition-all ${
                    copilotMode === mode
                      ? 'bg-[var(--vigil-accent)]/10 border-[var(--vigil-accent)]/30 text-[var(--vigil-accent)]'
                      : 'border-[var(--vigil-border)] text-[var(--vigil-muted)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)]'
                  }`}
                >
                  {mode}
                </button>
              ))}
            </div>
            <button
              onClick={runCopilot}
              disabled={copilotLoading}
              className="mb-4 px-4 py-2 rounded-xl text-sm font-bold bg-[var(--vigil-accent)] text-slate-950 hover:brightness-110 transition-all flex items-center gap-1.5 disabled:opacity-50"
            >
              <Play size={14} /> {copilotLoading ? 'Running...' : 'Run Copilot'}
            </button>
            {copilotResponse && (
              <div className="p-4 rounded-xl bg-black/30 border border-[var(--vigil-accent)]/15 text-sm text-[var(--vigil-text)] leading-relaxed max-h-[300px] overflow-y-auto">
                {copilotResponse}
              </div>
            )}
            {detail.copilot_history && detail.copilot_history.length > 0 && (
              <div className="mt-4 pt-4 border-t border-[var(--vigil-border)]">
                <div className="text-xs font-bold text-[var(--vigil-dim)] uppercase tracking-wider mb-2">History</div>
                {detail.copilot_history.map((h, i) => (
                  <div key={i} className="mb-2 p-3 rounded-lg bg-white/[0.02] border border-white/5">
                    <div className="text-xs font-semibold text-[var(--vigil-accent)] mb-1">{h.mode}</div>
                    <div className="text-xs text-[var(--vigil-muted)] line-clamp-2">{h.response}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Right column */}
        <div className="flex flex-col gap-6">
          {/* Actions */}
          <div className="p-6 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
            <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-4 flex items-center gap-2"><Zap size={16} className="text-[var(--vigil-accent)]" /> Actions</h3>
            <div className="grid grid-cols-2 gap-2 mb-4">
              {['acknowledge', 'assign', 'reroute', 'override', 'resolve'].map(action => (
                <button
                  key={action}
                  onClick={() => takeAction(action)}
                  disabled={actionLoading}
                  className="px-3 py-2.5 rounded-xl text-xs font-semibold border border-[var(--vigil-border)] bg-white/[0.02] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:bg-[var(--vigil-accent)]/5 hover:text-[var(--vigil-accent)] transition-all disabled:opacity-50"
                >
                  {action}
                </button>
              ))}
            </div>
            <textarea
              value={actionNote}
              onChange={e => setActionNote(e.target.value)}
              placeholder="Add a note..."
              className="w-full px-3 py-2.5 rounded-xl bg-black/20 border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)] resize-y min-h-[80px]"
            />
          </div>

          {/* Replay */}
          {replay && (
            <div className="p-6 rounded-2xl border border-[var(--vigil-green)]/20 bg-gradient-to-br from-[var(--vigil-green)]/[0.04] to-[var(--vigil-card)]">
              <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-4 flex items-center gap-2"><ShieldCheck size={16} className="text-[var(--vigil-green)]" /> Replay</h3>
              <div className="space-y-2 mb-4">
                <div className="font-mono text-[10px] text-[var(--vigil-green)] break-all p-2.5 rounded-lg bg-black/30 border border-[var(--vigil-green)]/10">
                  root: {replay.merkle_root?.slice(0, 32)}...
                </div>
                {replay.proof?.map((p: string, i: number) => (
                  <div key={i} className="font-mono text-[10px] text-[var(--vigil-accent)] break-all p-2.5 rounded-lg bg-black/30 border border-[var(--vigil-accent)]/10">
                    proof[{i}]: {p.slice(0, 32)}...
                  </div>
                ))}
              </div>
              <div className="inline-flex items-center gap-1.5 px-3 py-2 rounded-lg bg-[var(--vigil-green)]/10 border border-[var(--vigil-green)]/20 text-[var(--vigil-green)] text-xs font-bold font-mono mb-4">
                <CheckCircle2 size={14} />
                {replay.verification || 'Valid'}
              </div>
              <div className="flex flex-col gap-2">
                <button onClick={() => api.exportIncidentJson(id)} className="w-full px-3 py-2 rounded-xl text-xs font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all flex items-center justify-center gap-1.5">
                  <Download size={12} /> Export JSON
                </button>
                <button onClick={() => api.exportIncidentPdf(id)} className="w-full px-3 py-2 rounded-xl text-xs font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all flex items-center justify-center gap-1.5">
                  <Download size={12} /> Export PDF
                </button>
                <button onClick={async () => {
                  const res = await apiFetchMock(`/api/incidents/${id}/notify/mailto`)
                  if (res.mailto) window.location.href = res.mailto
                }} className="w-full px-3 py-2 rounded-xl text-xs font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all flex items-center justify-center gap-1.5">
                  <Mail size={12} /> Notify
                </button>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

async function apiFetchMock(path: string) {
  try {
    const token = sessionStorage.getItem('vigil_token') || ''
    const headers: Record<string, string> = {}
    if (token) headers['Authorization'] = `Bearer ${token}`
    const res = await fetch(path, { headers })
    if (!res.ok) throw new Error()
    return await res.json()
  } catch {
    return { mailto: 'mailto:?subject=Vigil+Incident&body=Incident+details' }
  }
}

/* ─── HEALTH VIEW ─── */
function HealthView() {
  const [health, setHealth] = useState<any>({})
  const [status, setStatus] = useState<any>({})
  const [copilotStatus, setCopilotStatus] = useState<any>({})
  const [slack, setSlack] = useState<any>({})
  const [slackUrl, setSlackUrl] = useState('')
  const [slackMsg, setSlackMsg] = useState('')

  async function load() {
    setHealth(await api.getHealth())
    setStatus(await api.getStatus())
    try {
      const res = await fetch('/api/copilot/status')
      setCopilotStatus(res.ok ? await res.json() : { mode: 'local', provider: 'builtin' })
    } catch {
      setCopilotStatus({ mode: 'local', provider: 'builtin' })
    }
    setSlack(await api.getSlackStatus())
  }

  useEffect(() => { load() }, [])

  async function saveSlack() {
    const res = await api.saveSlackWebhook(slackUrl)
    setSlackMsg(JSON.stringify(res))
    setSlack(await api.getSlackStatus())
  }

  async function testSlack() {
    const res = await api.testSlackWebhook()
    setSlackMsg(JSON.stringify(res))
  }

  async function clearSlack() {
    await api.saveSlackWebhook('')
    setSlackUrl('')
    setSlackMsg('Cleared')
    setSlack(await api.getSlackStatus())
  }

  return (
    <div>
      <div className="mb-6">
        <div className="text-xs font-mono text-[var(--vigil-dim)] mb-1">vigil:// / <span className="text-[var(--vigil-accent)]">health</span></div>
        <h1 className="text-3xl font-extrabold tracking-tight text-[var(--vigil-text)] mb-1">System Health</h1>
        <p className="text-sm text-[var(--vigil-muted)]">Real-time monitoring and operational metrics</p>
      </div>

      <HealthStrip health={health} />

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="p-5 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
          <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-3 flex items-center gap-2"><Activity size={16} className="text-[var(--vigil-accent)]" /> Last Ingest</h3>
          <div className="font-mono text-sm text-[var(--vigil-muted)]">{health.last_ingest || '—'}</div>
        </div>
        <div className="p-5 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
          <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-3 flex items-center gap-2"><Settings size={16} className="text-[var(--vigil-accent)]" /> Node Status</h3>
          <pre className="font-mono text-xs text-[var(--vigil-muted)] bg-black/30 p-3 rounded-xl overflow-auto max-h-[200px]">{JSON.stringify(status, null, 2)}</pre>
        </div>
        <div className="p-5 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
          <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-3 flex items-center gap-2"><BrainCircuit size={16} className="text-[var(--vigil-accent)]" /> Copilot Profile</h3>
          <pre className="font-mono text-xs text-[var(--vigil-muted)] bg-black/30 p-3 rounded-xl overflow-auto max-h-[200px]">{JSON.stringify(copilotStatus, null, 2)}</pre>
        </div>
        <div className="p-5 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
          <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-3 flex items-center gap-2"><Send size={16} className="text-[var(--vigil-accent)]" /> Slack Integration</h3>
          <p className="text-xs text-[var(--vigil-dim)] mb-3">{slack.configured ? `Configured · ${slack.masked_url || ''}` : 'No webhook configured'}</p>
          <div className="flex flex-wrap gap-2 mb-3">
            <input value={slackUrl} onChange={e => setSlackUrl(e.target.value)} placeholder="https://hooks.slack.com/..." className="flex-1 min-w-[200px] px-3 py-2 rounded-xl bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)]" />
            <button onClick={saveSlack} className="px-4 py-2 rounded-xl text-xs font-bold bg-[var(--vigil-accent)] text-slate-950 hover:brightness-110 transition-all">Save</button>
            <button onClick={testSlack} className="px-4 py-2 rounded-xl text-xs font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all">Test</button>
            <button onClick={clearSlack} className="px-4 py-2 rounded-xl text-xs font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-red-500/50 hover:text-red-400 transition-all flex items-center gap-1"><Trash2 size={12} /> Clear</button>
          </div>
          {slackMsg && <pre className="font-mono text-[11px] text-[var(--vigil-dim)] bg-black/20 p-2 rounded-lg">{slackMsg}</pre>}
        </div>
      </div>
    </div>
  )
}

/* ─── TELEMETRY VIEW ─── */
function TelemetryView() {
  const [sensors, setSensors] = useState<string[]>([])
  const [selected, setSelected] = useState('')
  const [history, setHistory] = useState<api.DataPoint[]>([])
  const [analytics, setAnalytics] = useState<any>({})
  const [chartStatus, setChartStatus] = useState('')
  const [actionMsg, setActionMsg] = useState('')
  const [writeVal, setWriteVal] = useState('')
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const chartRef = useRef<ChartType | null>(null)

  async function loadSensors() {
    const list = await api.listSensors()
    setSensors(list)
    if (list.length && !selected) {
      setSelected(list[0])
    }
  }

  async function loadChart() {
    if (!selected) return
    setChartStatus('Loading...')
    const data = await api.getSensorHistory(selected)
    const pts = data.datapoints || []
    setHistory(pts)
    setChartStatus(pts.length ? '' : 'No datapoints. Run seed or simulate.')

    if (chartRef.current) {
      chartRef.current.destroy()
      chartRef.current = null
    }
    if (!canvasRef.current || !pts.length) return

    const isLight = document.documentElement.getAttribute('data-theme') === 'light'
    const textColor = isLight ? '#0f172a' : '#e2e8f0'
    const gridColor = isLight ? 'rgba(15,23,42,0.08)' : 'rgba(226,232,240,0.08)'
    const lineColor = isLight ? '#d97706' : '#f59e0b'
    const fillColor = isLight ? 'rgba(217,119,6,0.15)' : 'rgba(245,158,11,0.12)'

    const labels = pts.map((p: api.DataPoint) => {
      const d = new Date(p.x)
      return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit', second: '2-digit' })
    })
    const values = pts.map((p: api.DataPoint) => p.y)

    chartRef.current = new Chart(canvasRef.current, {
      type: 'line',
      data: {
        labels,
        datasets: [{
          label: selected,
          data: values,
          borderColor: lineColor,
          backgroundColor: fillColor,
          fill: true,
          tension: 0.25,
          pointRadius: pts.length > 120 ? 0 : 2,
          pointHoverRadius: 4,
        }],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { intersect: false, mode: 'index' },
        scales: {
          x: { ticks: { color: textColor, maxRotation: 0, maxTicksLimit: 14 }, grid: { color: gridColor } },
          y: { ticks: { color: textColor }, grid: { color: gridColor } },
        },
        plugins: {
          legend: { labels: { color: textColor } },
        },
      },
    })

    const an = await api.getSensorAnalytics(selected)
    setAnalytics(an)
  }

  useEffect(() => {
    loadSensors()
  }, [])

  useEffect(() => {
    if (selected) loadChart()
  }, [selected])

  async function simulate() {
    if (!selected) return
    const res = await api.simulateSensor(selected)
    setActionMsg(JSON.stringify(res, null, 2))
    await loadChart()
  }

  async function write() {
    if (!selected) return
    const val = parseFloat(writeVal)
    if (isNaN(val)) return
    const res = await api.writeSensor(selected, val)
    setActionMsg(JSON.stringify(res, null, 2))
    await loadChart()
  }

  return (
    <div>
      <div className="mb-6">
        <div className="text-xs font-mono text-[var(--vigil-dim)] mb-1">vigil:// / <span className="text-[var(--vigil-accent)]">telemetry</span></div>
        <h1 className="text-3xl font-extrabold tracking-tight text-[var(--vigil-text)] mb-1">Sensor Trends</h1>
        <p className="text-sm text-[var(--vigil-muted)]">Live series from sensor history</p>
      </div>

      <div className="p-5 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)] mb-6">
        <div className="flex flex-wrap items-center gap-3 mb-4">
          <label className="text-sm text-[var(--vigil-muted)]">Sensor</label>
          <select value={selected} onChange={e => setSelected(e.target.value)} className="px-3 py-2 rounded-xl bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm font-mono focus:outline-none focus:border-[var(--vigil-accent)]">
            {sensors.map(s => <option key={s} value={s}>{s}</option>)}
          </select>
          <button onClick={loadChart} className="px-4 py-2 rounded-xl text-xs font-bold bg-[var(--vigil-accent)] text-slate-950 hover:brightness-110 transition-all flex items-center gap-1"><RefreshCw size={12} /> Refresh</button>
          <span className="text-xs font-mono text-[var(--vigil-dim)]">{history.length} datapoints</span>
        </div>
        {chartStatus && <p className="text-xs font-mono text-[var(--vigil-muted)] mb-2">{chartStatus}</p>}
        <div className="relative w-full h-[min(420px,52vh)] min-h-[280px]">
          <canvas ref={canvasRef} />
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mb-6">
        <div className="p-5 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
          <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-3">Analytics</h3>
          <pre className="font-mono text-xs text-[var(--vigil-muted)] bg-black/30 p-3 rounded-xl overflow-auto max-h-[200px]">{JSON.stringify(analytics, null, 2)}</pre>
        </div>
        <div className="p-5 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
          <h3 className="text-sm font-bold text-[var(--vigil-text)] mb-3">Ingest & Export</h3>
          <div className="flex flex-wrap gap-2 mb-3">
            <button onClick={simulate} className="px-4 py-2 rounded-xl text-xs font-bold bg-[var(--vigil-accent)] text-slate-950 hover:brightness-110 transition-all">Simulate</button>
            <input value={writeVal} onChange={e => setWriteVal(e.target.value)} placeholder="value" className="px-3 py-2 rounded-xl bg-white/[0.04] border border-[var(--vigil-border)] text-[var(--vigil-text)] text-sm placeholder-[var(--vigil-dim)] focus:outline-none focus:border-[var(--vigil-accent)] w-24" />
            <button onClick={write} className="px-4 py-2 rounded-xl text-xs font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all">Write</button>
          </div>
          {actionMsg && <pre className="font-mono text-[11px] text-[var(--vigil-dim)] bg-black/20 p-2 rounded-lg">{actionMsg}</pre>}
        </div>
      </div>
    </div>
  )
}

/* ─── MESH VIEW ─── */
function MeshView() {
  const [topology, setTopology] = useState<any>({})

  async function load() {
    setTopology(await api.getMeshTopology())
  }

  useEffect(() => { load() }, [])

  return (
    <div>
      <div className="mb-6">
        <div className="text-xs font-mono text-[var(--vigil-dim)] mb-1">vigil:// / <span className="text-[var(--vigil-accent)]">mesh</span></div>
        <h1 className="text-3xl font-extrabold tracking-tight text-[var(--vigil-text)] mb-1">Mesh Topology</h1>
        <p className="text-sm text-[var(--vigil-muted)]">Live snapshot from gossip engine</p>
      </div>
      <div className="p-5 rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)]">
        <div className="flex justify-between items-center mb-4">
          <h3 className="text-sm font-bold text-[var(--vigil-text)]">Topology JSON</h3>
          <button onClick={load} className="px-4 py-2 rounded-xl text-xs font-bold bg-[var(--vigil-accent)] text-slate-950 hover:brightness-110 transition-all flex items-center gap-1"><RefreshCw size={12} /> Refresh</button>
        </div>
        <pre className="font-mono text-xs text-[var(--vigil-muted)] bg-black/30 p-4 rounded-xl overflow-auto max-h-[60vh]">{JSON.stringify(topology, null, 2)}</pre>
      </div>
    </div>
  )
}

/* ─── MAIN DASHBOARD ─── */
export default function Dashboard() {
  const { theme, toggle } = useTheme()
  const [activeView, setActiveView] = useState('incidents')
  const [selectedIncident, setSelectedIncident] = useState<string | null>(null)
  const [counts, setCounts] = useState({ incidents: 0, open: 0 })
  const [refreshKey, setRefreshKey] = useState(0)

  const location = useLocation()

  useEffect(() => {
    const hash = location.hash.replace('#', '')
    if (hash.startsWith('detail/')) {
      const id = hash.replace('detail/', '')
      setSelectedIncident(id)
      setActiveView('detail')
    } else if (hash) {
      setActiveView(hash)
    }
  }, [location])

  async function refreshCounts() {
    const data = await api.listIncidents()
    setCounts({ incidents: data.length, open: data.filter((i: Incident) => i.status === 'open').length })
  }

  useEffect(() => {
    refreshCounts()
    const iv = setInterval(refreshCounts, 5000)
    return () => clearInterval(iv)
  }, [refreshKey])

  async function runDetection() {
    const res = await api.runDetection()
    alert(`Detection complete! Created: ${res.created_incidents?.length || 0}, Processed: ${res.events_processed}`)
    setRefreshKey(k => k + 1)
  }

  function handleSelectIncident(id: string) {
    setSelectedIncident(id)
    setActiveView('detail')
    window.location.hash = `detail/${id}`
  }

  function handleBack() {
    setSelectedIncident(null)
    setActiveView('incidents')
    window.location.hash = 'incidents'
  }

  return (
    <div className="min-h-[100dvh] flex flex-col bg-[var(--vigil-bg)] text-[var(--vigil-text)]">
      {/* Top nav */}
      <nav className="sticky top-0 z-50 border-b border-[var(--vigil-border)] bg-[var(--vigil-bg)]/95 backdrop-blur-xl">
        <div className="flex items-center justify-between px-6 py-3">
          <div className="flex items-center gap-4">
            <Link to="/" className="flex items-center gap-2 text-[var(--vigil-text)] font-bold no-underline">
              <svg viewBox="0 0 28 28" width={22} height={22} fill="none">
                <path d="M14 2L2 26L14 20L26 26L14 2Z" fill="#f59e0b" />
                <path d="M14 2L14 20L2 26L14 2Z" fill="#d97706" />
              </svg>
              Vigil
            </Link>
            <span className="text-[var(--vigil-dim)] text-xs font-mono hidden sm:inline">Operational Intelligence</span>
          </div>
          <div className="flex items-center gap-3 flex-wrap">
            <LoginBar onRefresh={() => setRefreshKey(k => k + 1)} />
            <button onClick={toggle} className="px-3 py-1.5 rounded-lg text-xs font-semibold border border-[var(--vigil-border)] text-[var(--vigil-muted)] hover:text-[var(--vigil-accent)] transition-all">
              {theme === 'dark' ? 'Light' : 'Dark'}
            </button>
            <button onClick={runDetection} className="px-4 py-1.5 rounded-lg text-xs font-bold bg-[var(--vigil-accent)] text-slate-950 hover:brightness-110 transition-all flex items-center gap-1.5">
              <Play size={14} /> Detect
            </button>
          </div>
        </div>
      </nav>

      <div className="flex flex-1 overflow-hidden">
        <Sidebar active={activeView} counts={counts} />
        <main className="flex-1 overflow-y-auto p-6">
          {activeView === 'incidents' && <IncidentListView onSelect={handleSelectIncident} />}
          {activeView === 'detail' && selectedIncident && <IncidentDetailView id={selectedIncident} onBack={handleBack} />}
          {activeView === 'health' && <HealthView />}
          {activeView === 'telemetry' && <TelemetryView />}
          {activeView === 'mesh' && <MeshView />}
        </main>
      </div>
    </div>
  )
}
