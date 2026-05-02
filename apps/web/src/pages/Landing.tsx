import { useEffect, useRef, useState } from 'react'
import { Link } from 'react-router-dom'
import { gsap } from 'gsap'
import { ScrollTrigger } from 'gsap/ScrollTrigger'
import {
  Activity,
  BrainCircuit,
  CheckCircle2,
  Database,
  GitBranch,
  LayoutDashboard,
  Play,
  RefreshCw,
  ShieldCheck,
  Zap,
} from 'lucide-react'
import * as api from '../lib/api'

gsap.registerPlugin(ScrollTrigger)

/* ─── THEME TOGGLE HOOK ─── */
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

/* ─── NAV ─── */
function Nav({ theme, onToggle }: { theme: string; onToggle: () => void }) {
  const [scrolled, setScrolled] = useState(false)
  useEffect(() => {
    const handler = () => setScrolled(window.scrollY > 40)
    window.addEventListener('scroll', handler, { passive: true })
    return () => window.removeEventListener('scroll', handler)
  }, [])

  return (
    <nav
      className={`fixed top-4 left-1/2 -translate-x-1/2 z-50 transition-all duration-500 ${
        scrolled
          ? 'bg-[var(--vigil-bg)]/80 backdrop-blur-xl shadow-2xl border border-[var(--vigil-border)]'
          : 'bg-transparent'
      }`}
      style={{ borderRadius: 9999, padding: '12px 28px' }}
    >
      <div className="flex items-center gap-8">
        <Link to="/" className="flex items-center gap-2.5 text-[var(--vigil-text)] font-bold text-lg tracking-tight no-underline">
          <svg viewBox="0 0 28 28" width={24} height={24} fill="none" aria-hidden="true">
            <path d="M14 2L2 26L14 20L26 26L14 2Z" fill="#f59e0b" />
            <path d="M14 2L14 20L2 26L14 2Z" fill="#d97706" />
          </svg>
          Vigil
        </Link>
        <div className="hidden md:flex items-center gap-6 text-sm text-[var(--vigil-muted)]">
          <a href="#features" className="hover:text-[var(--vigil-text)] transition-colors no-underline">Platform</a>
          <a href="#workflow" className="hover:text-[var(--vigil-text)] transition-colors no-underline">Workflow</a>
          <a href="#integrity" className="hover:text-[var(--vigil-text)] transition-colors no-underline">Trust</a>
          <Link to="/dashboard" className="hover:text-[var(--vigil-text)] transition-colors no-underline">Dashboard</Link>
        </div>
        <div className="flex items-center gap-3">
          <button
            type="button"
            onClick={onToggle}
            className="text-xs text-[var(--vigil-muted)] bg-transparent border-none cursor-pointer hover:text-[var(--vigil-accent)] transition-colors"
          >
            {theme === 'dark' ? 'Light' : 'Dark'}
          </button>
          <Link
            to="/dashboard"
            className="px-5 py-2 rounded-full text-sm font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] no-underline hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all"
          >
            Open Dashboard
          </Link>
        </div>
      </div>
    </nav>
  )
}

/* ─── HERO ─── */
function Hero() {
  const heroRef = useRef<HTMLElement>(null)
  const titleRef = useRef<HTMLHeadingElement>(null)
  const dashRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!titleRef.current || !dashRef.current) return
    const ctx = gsap.context(() => {
      gsap.from(titleRef.current, {
        y: 60,
        opacity: 0,
        duration: 1.2,
        ease: 'power3.out',
        delay: 0.2,
      })
      gsap.from(dashRef.current, {
        y: 80,
        opacity: 0,
        scale: 0.9,
        duration: 1.4,
        ease: 'power3.out',
        delay: 0.5,
      })
    }, heroRef)
    return () => ctx.revert()
  }, [])

  return (
    <section ref={heroRef} className="relative min-h-[100dvh] flex items-center overflow-hidden pt-24 pb-16">
      {/* Background ambient */}
      <div className="absolute inset-0 pointer-events-none">
        <div
          className="absolute top-[-20%] right-[-10%] w-[700px] h-[700px] rounded-full opacity-[0.06]"
          style={{ background: 'radial-gradient(circle, #f59e0b, transparent 70%)' }}
        />
        <div
          className="absolute bottom-[-10%] left-[-10%] w-[500px] h-[500px] rounded-full opacity-[0.04]"
          style={{ background: 'radial-gradient(circle, #10b981, transparent 70%)' }}
        />
      </div>

      <div className="relative z-10 w-full max-w-7xl mx-auto px-6 grid grid-cols-1 lg:grid-cols-2 gap-12 lg:gap-8 items-center">
        {/* Left: Text */}
        <div>
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full text-xs font-semibold tracking-wide uppercase border border-[var(--vigil-accent)]/20 bg-[var(--vigil-accent)]/5 text-[var(--vigil-accent)] mb-8">
            <span className="w-1.5 h-1.5 rounded-full bg-[var(--vigil-accent)] animate-pulse-dot" />
            Local-first. Zero recurring cost. Merkle-backed.
          </div>
          <h1
            ref={titleRef}
            className="max-w-6xl text-[clamp(3rem,5.5vw,5.5rem)] font-extrabold leading-[0.95] tracking-[-0.04em] text-[var(--vigil-text)] mb-8"
          >
            Turn industrial{' '}
            <span className="relative inline-block align-middle mx-1">
              <span
                className="inline-block w-20 h-9 md:w-28 md:h-10 rounded-full align-middle bg-cover bg-center"
                style={{ backgroundImage: 'url(https://picsum.photos/seed/factoryfloor/1920/1080)' }}
              />
            </span>{' '}
            chaos into operational intelligence.
          </h1>
          <p className="text-lg text-[var(--vigil-muted)] max-w-xl leading-relaxed mb-10">
            Vigil ingests noisy machine logs, maintenance tickets, and operator notes — then detects explainable incidents, recommends actions, and preserves tamper-evident replay for every decision.
          </p>
          <div className="flex flex-wrap gap-4">
            <Link
              to="/dashboard"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full text-base font-bold bg-[var(--vigil-accent)] text-slate-950 no-underline hover:translate-y-[-2px] transition-all shadow-lg"
              style={{ boxShadow: '0 0 40px var(--vigil-accent-glow)' }}
            >
              <LayoutDashboard size={18} />
              Open Dashboard
            </Link>
            <a
              href="#workflow"
              className="inline-flex items-center gap-2 px-7 py-3.5 rounded-full text-base font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] no-underline hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all"
            >
              <Play size={18} />
              See the Workflow
            </a>
          </div>
        </div>

        {/* Right: Dashboard mock */}
        <div ref={dashRef} className="relative">
          <div
            className="rounded-2xl border border-[var(--vigil-accent)]/15 overflow-hidden"
            style={{
              background: 'linear-gradient(145deg, rgba(17,24,39,0.95), var(--vigil-card))',
              boxShadow: '0 0 80px rgba(245,158,11,0.1)',
            }}
          >
            <div className="flex items-center justify-between px-5 py-3.5 border-b border-[var(--vigil-border)] bg-black/30">
              <div className="flex gap-2">
                <span className="w-3 h-3 rounded-full bg-red-500" />
                <span className="w-3 h-3 rounded-full bg-amber-500" />
                <span className="w-3 h-3 rounded-full bg-emerald-500" />
              </div>
              <span className="font-mono text-[11px] text-[var(--vigil-muted)]">vigil://ops/incidents/live</span>
              <div className="flex gap-2">
                <span className="px-2.5 py-1 rounded-md bg-[var(--vigil-accent)]/10 text-[var(--vigil-accent)] text-[10px] font-mono">24h</span>
              </div>
            </div>
            <div className="p-5 grid grid-cols-[160px_1fr] gap-4 min-h-[320px]">
              <div className="flex flex-col gap-2">
                {['ontario-line1-temp', 'ontario-line1-vibration', 'ontario-line2-temp', 'detroit-press-temp', 'detroit-press-vibration'].map((s, i) => (
                  <div
                    key={s}
                    className={`px-3 py-2.5 rounded-lg text-[11px] font-mono border ${
                      i === 0
                        ? 'border-[var(--vigil-accent)]/25 bg-[var(--vigil-accent)]/5 text-[var(--vigil-accent)]'
                        : 'border-white/5 bg-white/[0.02] text-[var(--vigil-muted)]'
                    }`}
                  >
                    {s}
                  </div>
                ))}
              </div>
              <div className="flex flex-col gap-3">
                <div className="flex-1 rounded-xl bg-white/[0.02] border border-white/5 p-4 relative overflow-hidden">
                  <svg className="absolute bottom-0 left-0 right-0 h-full" viewBox="0 0 400 160" preserveAspectRatio="none">
                    <defs>
                      <linearGradient id="heroG1" x1="0" y1="0" x2="1" y2="0">
                        <stop offset="0%" stopColor="#f59e0b" />
                        <stop offset="100%" stopColor="#d97706" />
                      </linearGradient>
                      <linearGradient id="heroFill" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor="rgba(245,158,11,0.15)" />
                        <stop offset="100%" stopColor="transparent" />
                      </linearGradient>
                    </defs>
                    <path d="M0,120 Q40,110 80,95 T160,80 T240,50 T320,70 T400,30 V160 H0Z" fill="url(#heroFill)" />
                    <path d="M0,120 Q40,110 80,95 T160,80 T240,50 T320,70 T400,30" fill="none" stroke="url(#heroG1)" strokeWidth="2.5" />
                    <circle cx="320" cy="70" r="4" fill="#ef4444" />
                    <circle cx="320" cy="70" r="8" fill="none" stroke="rgba(239,68,68,0.3)" strokeWidth="1.5" />
                  </svg>
                </div>
                <div className="grid grid-cols-3 gap-2">
                  {[
                    { label: 'Temp Spike', level: 'critical', time: '2m ago' },
                    { label: 'Vibration', level: 'high', time: '8m ago' },
                    { label: 'Cascade', level: 'medium', time: '14m ago' },
                  ].map(item => (
                    <div
                      key={item.label}
                      className={`px-3 py-2.5 rounded-lg text-[11px] border ${
                        item.level === 'critical'
                          ? 'border-red-500/30 bg-red-500/5'
                          : item.level === 'high'
                          ? 'border-amber-500/30 bg-amber-500/5'
                          : 'border-[var(--vigil-accent)]/20 bg-[var(--vigil-accent)]/5'
                      }`}
                    >
                      <div className="font-bold text-[12px] mb-1 text-[var(--vigil-text)]">{item.label}</div>
                      <div className="text-[var(--vigil-muted)]">{item.level} · {item.time}</div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}

/* ─── PULSE BAR ─── */
function PulseBar() {
  const [health, setHealth] = useState({ events_last_hour: 0, incidents_open: 0, data_quality: '—', mesh_nodes: 0, last_ingest: '—' })
  const [status, setStatus] = useState({ node_id: '—', stats: { total_records: 0 } })

  useEffect(() => {
    let active = true
    async function load() {
      const h = await api.getHealth()
      const s = await api.getStatus()
      if (!active) return
      setHealth(h)
      setStatus(s)
    }
    load()
    const iv = setInterval(load, 5000)
    return () => { active = false; clearInterval(iv) }
  }, [])

  return (
    <section className="py-8 px-6">
      <div className="max-w-6xl mx-auto flex flex-wrap items-center justify-center gap-x-6 gap-y-3 px-8 py-4 rounded-full border border-[var(--vigil-border)] bg-[var(--vigil-card)]/60 backdrop-blur-md font-mono text-xs text-[var(--vigil-muted)]">
        <div className="flex items-center gap-2 text-[var(--vigil-accent)] font-bold tracking-wider">
          <span className="w-1.5 h-1.5 rounded-full bg-[var(--vigil-accent)] animate-pulse-dot" />
          LIVE PULSE
        </div>
        <span className="text-[var(--vigil-dim)]">|</span>
        <div>EVENTS/HOUR: <span className="text-[var(--vigil-text)] font-semibold">{health.events_last_hour}</span></div>
        <span className="text-[var(--vigil-dim)]">|</span>
        <div>DATA QUALITY: <span className="text-[var(--vigil-text)] font-semibold">{health.data_quality}</span></div>
        <span className="text-[var(--vigil-dim)]">|</span>
        <div>OPEN: <span className="text-[var(--vigil-text)] font-semibold">{health.incidents_open}</span></div>
        <span className="text-[var(--vigil-dim)]">|</span>
        <div>LAST INGEST: <span className="text-[var(--vigil-text)] font-semibold">{health.last_ingest ? new Date(health.last_ingest).toLocaleTimeString() : '—'}</span></div>
        <span className="text-[var(--vigil-dim)]">|</span>
        <div>NODE: <span className="text-[var(--vigil-text)] font-semibold">{status.node_id}</span></div>
        <span className="text-[var(--vigil-dim)]">|</span>
        <div>RECORDS: <span className="text-[var(--vigil-text)] font-semibold">{status.stats?.total_records?.toLocaleString() || '—'}</span></div>
      </div>
    </section>
  )
}

/* ─── BENTO FEATURES ─── */
function Features() {
  const sectionRef = useRef<HTMLElement>(null)

  useEffect(() => {
    if (!sectionRef.current) return
    const ctx = gsap.context(() => {
      gsap.from('.bento-card', {
        y: 50,
        opacity: 0,
        duration: 0.8,
        stagger: 0.1,
        ease: 'power3.out',
        scrollTrigger: {
          trigger: sectionRef.current,
          start: 'top 75%',
        },
      })
    }, sectionRef)
    return () => ctx.revert()
  }, [])

  const cards = [
    {
      title: 'Explainable Incidents',
      desc: 'Every incident carries a rule-firing timeline, suspected cause, and confidence score. No black-box anomalies.',
      icon: <BrainCircuit size={22} />,
      span: 'col-span-2 row-span-2',
      image: 'https://picsum.photos/seed/industrialbrain/800/600',
    },
    {
      title: 'Actionable Recommendations',
      desc: 'The system recommends next actions and records the operator decision in the same audit ledger.',
      icon: <Zap size={22} />,
      span: 'col-span-2 row-span-1',
    },
    {
      title: 'Merkle-Backed Audit',
      desc: 'Every decision is cryptographically sealed. Replay payloads include Merkle roots and proof arrays.',
      icon: <ShieldCheck size={22} />,
      span: 'col-span-1 row-span-2',
    },
    {
      title: 'Local-First Storage',
      desc: 'Sled for immutable telemetry chains, SQLite for operational state. No cloud dependencies.',
      icon: <Database size={22} />,
      span: 'col-span-1 row-span-1',
    },
    {
      title: 'Messy-Data Fluent',
      desc: 'Built for real manufacturing: late arrivals, duplicates, out-of-order events, and conflicting notes.',
      icon: <CheckCircle2 size={22} />,
      span: 'col-span-2 row-span-1',
    },
  ]

  return (
    <section ref={sectionRef} id="features" className="py-32 md:py-48 px-6 relative">
      <div className="absolute inset-0 pointer-events-none">
        <div className="absolute top-0 left-1/4 w-[600px] h-[600px] rounded-full opacity-[0.03]" style={{ background: 'radial-gradient(circle, #f59e0b, transparent 70%)' }} />
      </div>
      <div className="max-w-6xl mx-auto relative z-10">
        <h2 className="text-[clamp(2.5rem,4vw,3.5rem)] font-extrabold tracking-[-0.03em] leading-tight text-[var(--vigil-text)] mb-6">
          A decision system, not a{' '}
          <span className="relative inline-block align-middle mx-1">
            <span
              className="inline-block w-16 h-8 md:w-20 md:h-9 rounded-full align-middle bg-cover bg-center"
              style={{ backgroundImage: 'url(https://picsum.photos/seed/dashboard/800/400)' }}
            />
          </span>{' '}
          dashboard.
        </h2>
        <p className="text-lg text-[var(--vigil-muted)] max-w-2xl leading-relaxed mb-16">
          Vigil compresses evidence, reasoning, and operator action into a single surface — so teams move from signal to decision without digging through disconnected tooling.
        </p>

        <div className="grid grid-cols-4 grid-rows-3 gap-4 grid-flow-dense auto-rows-[180px]">
          {cards.map((card, i) => (
            <div
              key={i}
              className={`bento-card group relative overflow-hidden rounded-2xl border border-[var(--vigil-border)] bg-gradient-to-br from-[var(--vigil-card)] to-[var(--vigil-bg2)] p-6 flex flex-col justify-between transition-all duration-500 hover:border-[var(--vigil-accent)]/30 ${card.span}`}
            >
              {card.image && (
                <div className="absolute inset-0 opacity-0 group-hover:opacity-20 transition-opacity duration-700">
                  <img src={card.image} alt="" className="w-full h-full object-cover grayscale contrast-125" />
                </div>
              )}
              <div className="relative z-10">
                <div className="w-11 h-11 rounded-xl flex items-center justify-center bg-[var(--vigil-accent)]/10 border border-[var(--vigil-accent)]/20 text-[var(--vigil-accent)] mb-4">
                  {card.icon}
                </div>
                <h3 className="text-lg font-bold text-[var(--vigil-text)] mb-2">{card.title}</h3>
                <p className="text-sm text-[var(--vigil-muted)] leading-relaxed">{card.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}

/* ─── INFINITE MARQUEE ─── */
function Marquee() {
  const items = [
    'Rust + Axum',
    'SQLite + Sled',
    'Merkle DAG',
    'WebSocket Live',
    'Deterministic Detection',
    'Zero Cloud Cost',
    'Operator Actions',
    'Replay Native',
    'Local-First',
    'Manufacturing Ops',
  ]
  const doubled = [...items, ...items]

  return (
    <section className="py-16 overflow-hidden border-y border-[var(--vigil-border)]">
      <div className="flex animate-marquee whitespace-nowrap">
        {doubled.map((item, i) => (
          <span key={i} className="mx-8 text-2xl md:text-4xl font-extrabold tracking-tight text-[var(--vigil-border)] select-none">
            {item}
          </span>
        ))}
      </div>
    </section>
  )
}

/* ─── WORKFLOW (Scroll Pinning) ─── */
function Workflow() {
  const sectionRef = useRef<HTMLElement>(null)
  const leftRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!sectionRef.current || !leftRef.current) return
    const mm = window.matchMedia('(min-width: 1024px)')
    if (!mm.matches) return

    const ctx = gsap.context(() => {
      ScrollTrigger.create({
        trigger: sectionRef.current,
        start: 'top top',
        end: 'bottom bottom',
        pin: leftRef.current,
        pinSpacing: false,
      })
    }, sectionRef)
    return () => ctx.revert()
  }, [])

  const steps = [
    { num: '01', title: 'Ingest', desc: 'Collect machine telemetry, maintenance tickets, and operator notes with no assumption that sources arrive clean or on time.' },
    { num: '02', title: 'Detect', desc: 'Correlate events across time windows and rules so the system surfaces incidents, not raw noise.' },
    { num: '03', title: 'Explain', desc: 'Show what fired, why it fired, and what evidence raised or lowered confidence — grounded in real data.' },
    { num: '04', title: 'Act', desc: 'Capture acknowledgment, assignment, reroute, override, or resolution inside the same surface.' },
    { num: '05', title: 'Replay', desc: 'Reconstruct the complete decision chain — reasoning, operator response, and Merkle-backed integrity verification.' },
  ]

  return (
    <section ref={sectionRef} id="workflow" className="py-32 md:py-48 px-6">
      <div className="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-16">
        <div ref={leftRef} className="lg:pt-32">
          <h2 className="text-[clamp(2.5rem,4vw,3.5rem)] font-extrabold tracking-[-0.03em] leading-tight text-[var(--vigil-text)] mb-6">
            From ingestion<br />to action in one flow.
          </h2>
          <p className="text-lg text-[var(--vigil-muted)] leading-relaxed max-w-md">
            Every step is traceable. Every decision is recorded. Every replay is verifiable.
          </p>
        </div>
        <div className="flex flex-col gap-6">
          {steps.map((step, i) => (
            <div
              key={i}
              className="group flex gap-5 p-6 rounded-2xl border border-[var(--vigil-border)] bg-white/[0.02] hover:border-[var(--vigil-accent)]/30 hover:bg-[var(--vigil-accent)]/[0.02] transition-all duration-300"
            >
              <div className="w-14 h-14 rounded-xl flex-shrink-0 flex items-center justify-center bg-gradient-to-br from-[var(--vigil-accent)]/15 to-[var(--vigil-accent)]/5 border border-[var(--vigil-accent)]/20 text-[var(--vigil-accent)] font-extrabold text-xl">
                {step.num}
              </div>
              <div>
                <h4 className="text-lg font-bold text-[var(--vigil-text)] mb-1">{step.title}</h4>
                <p className="text-sm text-[var(--vigil-muted)] leading-relaxed">{step.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}

/* ─── INTEGRITY (Scrubbing Text) ─── */
function Integrity() {
  const textRef = useRef<HTMLParagraphElement>(null)

  useEffect(() => {
    if (!textRef.current) return
    const ctx = gsap.context(() => {
      const words = textRef.current!.querySelectorAll('.scrub-word')
      gsap.fromTo(
        words,
        { opacity: 0.1 },
        {
          opacity: 1,
          stagger: 0.05,
          ease: 'none',
          scrollTrigger: {
            trigger: textRef.current,
            start: 'top 70%',
            end: 'bottom 40%',
            scrub: true,
          },
        }
      )
    })
    return () => ctx.revert()
  }, [])

  const manifesto = `Every decision is cryptographically sealed. Replay payloads combine timeline events, rule identifiers, reasoning text, Merkle roots, operator actions, and copilot history into one verifiable record. Shift supervisors do not need more telemetry. They need clarity, confidence, and a defensible path to action.`
  const words = manifesto.split(' ')

  return (
    <section id="integrity" className="py-32 md:py-48 px-6">
      <div className="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-16 items-center">
        <div>
          <h2 className="text-[clamp(2.5rem,4vw,3.5rem)] font-extrabold tracking-[-0.03em] leading-tight text-[var(--vigil-text)] mb-6">
            Every decision<br />is cryptographically sealed.
          </h2>
          <p ref={textRef} className="text-xl md:text-2xl font-medium leading-relaxed text-[var(--vigil-text)] mb-10">
            {words.map((w, i) => (
              <span key={i} className="scrub-word inline-block mr-[0.3em]">{w}</span>
            ))}
          </p>
          <div className="flex flex-col gap-4">
            {[
              { icon: <ShieldCheck size={18} />, title: 'Tamper-Evident Lineage', desc: 'Sled-backed immutable telemetry chains with Merkle verification for every data point.' },
              { icon: <Activity size={18} />, title: 'Operator Visibility', desc: 'Every note, override, and action is recorded in the same incident ledger with timestamps and actor identity.' },
              { icon: <RefreshCw size={18} />, title: 'Full Replay', desc: 'Reconstruct the exact sequence of events, rules fired, reasoning, and human decisions that led to any outcome.' },
            ].map((item, i) => (
              <div key={i} className="flex items-start gap-4">
                <div className="w-9 h-9 rounded-lg flex items-center justify-center bg-[var(--vigil-green)]/10 border border-[var(--vigil-green)]/20 text-[var(--vigil-green)] flex-shrink-0 mt-0.5">
                  {item.icon}
                </div>
                <div>
                  <div className="font-bold text-[var(--vigil-text)] text-sm mb-0.5">{item.title}</div>
                  <div className="text-sm text-[var(--vigil-muted)] leading-relaxed">{item.desc}</div>
                </div>
              </div>
            ))}
          </div>
        </div>
        <div
          className="rounded-2xl border border-[var(--vigil-green)]/20 p-8 relative overflow-hidden"
          style={{ background: 'linear-gradient(160deg, rgba(16,185,129,0.05), var(--vigil-card))' }}
        >
          <div className="font-mono text-xs text-[var(--vigil-green)] font-bold mb-1">Replay Verification</div>
          <div className="font-mono text-[11px] text-[var(--vigil-muted)] mb-6">Incident: temp_spike · ontario-line1</div>
          <div className="space-y-3">
            {[
              'merkle_root: 0x7a3f9c2e8b1d4f6a0e5c3b9d7f2a8e1c4b6d0f3a9e7c2b5d8f1a4e6c3b9d7f2a',
              'proof[0]: 0x3b9d7f2a8e1c4b6d0f3a9e7c2b5d8f1a4e6c3b9d7f2a7a3f9c2e8b1d4f6a0e5c',
              'proof[1]: 0x8e1c4b6d0f3a9e7c2b5d8f1a4e6c3b9d7f2a7a3f9c2e8b1d4f6a0e5c3b9d7f2a',
            ].map((hash, i) => (
              <div key={i} className="font-mono text-[11px] text-[var(--vigil-green)] break-all p-3 rounded-lg bg-black/30 border border-[var(--vigil-green)]/10">
                {hash}
              </div>
            ))}
          </div>
          <div className="mt-6 inline-flex items-center gap-2 px-4 py-2.5 rounded-xl bg-[var(--vigil-green)]/10 border border-[var(--vigil-green)]/20 text-[var(--vigil-green)] text-sm font-bold font-mono">
            <CheckCircle2 size={16} />
            Valid Merkle path - data untampered
          </div>
        </div>
      </div>
    </section>
  )
}

/* ─── HORIZONTAL ACCORDIONS ─── */
function Accordions() {
  const [openIndex, setOpenIndex] = useState<number | null>(0)
  const items = [
    {
      title: 'Ingest',
      subtitle: 'Multi-source telemetry',
      content: 'Machine PLC logs, maintenance tickets, and operator notes are ingested with tolerance for late arrivals, duplicates, and conflicting timestamps.',
      image: 'https://picsum.photos/seed/ingest/600/400',
    },
    {
      title: 'Detect',
      subtitle: 'Deterministic rules',
      content: 'Versioned detection rules fire against correlated event windows. No black boxes — every trigger is explainable with evidence.',
      image: 'https://picsum.photos/seed/detect/600/400',
    },
    {
      title: 'Explain',
      subtitle: 'Confidence & reasoning',
      content: 'Incident detail surfaces rule metadata, contributing signals, confidence scores, and severity justification.',
      image: 'https://picsum.photos/seed/explain/600/400',
    },
    {
      title: 'Act',
      subtitle: 'Operator decisions',
      content: 'Five operator actions: acknowledge, assign maintenance, reroute, override with justification, and resolve. All logged.',
      image: 'https://picsum.photos/seed/act/600/400',
    },
    {
      title: 'Replay',
      subtitle: 'Merkle verification',
      content: 'Reconstruct any incident decision chain. Verify Merkle roots and proof arrays to confirm data integrity.',
      image: 'https://picsum.photos/seed/replay/600/400',
    },
  ]

  return (
    <section className="py-32 md:py-48 px-6">
      <div className="max-w-6xl mx-auto">
        <h2 className="text-[clamp(2rem,3.5vw,3rem)] font-extrabold tracking-[-0.03em] text-[var(--vigil-text)] mb-12 text-center">
          The incident lifecycle
        </h2>
        <div className="flex flex-col md:flex-row gap-3 h-[500px]">
          {items.map((item, i) => (
            <div
              key={i}
              onClick={() => setOpenIndex(i)}
              className={`relative overflow-hidden rounded-2xl border border-[var(--vigil-border)] cursor-pointer transition-all duration-700 ease-out ${
                openIndex === i ? 'md:flex-[3]' : 'md:flex-[0.6]'
              }`}
              style={{ background: 'var(--vigil-card)' }}
            >
              <div className="absolute inset-0 opacity-30">
                <img src={item.image} alt="" className="w-full h-full object-cover grayscale contrast-125" />
              </div>
              <div className="absolute inset-0 bg-gradient-to-t from-[var(--vigil-bg)] via-[var(--vigil-bg)]/80 to-transparent" />
              <div className="relative z-10 h-full flex flex-col justify-end p-6">
                <div className={`text-[var(--vigil-accent)] font-mono text-xs font-bold mb-2 ${openIndex === i ? 'opacity-100' : 'opacity-60'}`}>
                  {item.subtitle}
                </div>
                <h3 className="text-2xl font-extrabold text-[var(--vigil-text)] mb-3">{item.title}</h3>
                <div
                  className={`overflow-hidden transition-all duration-500 ${
                    openIndex === i ? 'max-h-40 opacity-100' : 'max-h-0 opacity-0'
                  }`}
                >
                  <p className="text-sm text-[var(--vigil-muted)] leading-relaxed max-w-md">{item.content}</p>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  )
}

/* ─── CTA ─── */
function Cta() {
  return (
    <section className="py-32 md:py-48 px-6">
      <div className="max-w-5xl mx-auto">
        <div
          className="relative overflow-hidden rounded-3xl border border-[var(--vigil-border)] px-8 py-20 md:px-16 md:py-24 text-center"
          style={{ background: 'linear-gradient(160deg, var(--vigil-card), var(--vigil-bg2))' }}
        >
          <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[600px] h-[600px] rounded-full opacity-[0.05] pointer-events-none" style={{ background: 'radial-gradient(circle, #f59e0b, transparent 70%)' }} />
          <h2 className="relative z-10 text-[clamp(2rem,4vw,3.5rem)] font-extrabold tracking-[-0.03em] text-[var(--vigil-text)] mb-5">
            Ready to close the loop?
          </h2>
          <p className="relative z-10 text-lg text-[var(--vigil-muted)] max-w-xl mx-auto mb-10 leading-relaxed">
            Vigil is local-first, replay-native, and built for manufacturing environments where trust matters. See it in action.
          </p>
          <div className="relative z-10 flex flex-wrap justify-center gap-4">
            <Link
              to="/dashboard"
              className="inline-flex items-center gap-2 px-8 py-4 rounded-full text-base font-bold bg-[var(--vigil-accent)] text-slate-950 no-underline hover:translate-y-[-2px] transition-all"
              style={{ boxShadow: '0 0 50px var(--vigil-accent-glow)' }}
            >
              <LayoutDashboard size={18} />
              Open Dashboard
            </Link>
            <a
              href="https://github.com/AngelP17/Vigil-ForgeMesh-"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 px-8 py-4 rounded-full text-base font-semibold border border-[var(--vigil-border)] text-[var(--vigil-text)] no-underline hover:border-[var(--vigil-accent)] hover:text-[var(--vigil-accent)] transition-all"
            >
              <GitBranch size={18} />
              View on GitHub
            </a>
          </div>
        </div>
      </div>
    </section>
  )
}

/* ─── FOOTER ─── */
function Footer() {
  return (
    <footer className="border-t border-[var(--vigil-border)] py-10 px-6">
      <div className="max-w-6xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4">
        <Link to="/" className="flex items-center gap-2.5 text-[var(--vigil-text)] font-bold no-underline">
          <svg viewBox="0 0 28 28" width={20} height={20} fill="none" aria-hidden="true">
            <path d="M14 2L2 26L14 20L26 26L14 2Z" fill="#f59e0b" />
            <path d="M14 2L14 20L2 26L14 2Z" fill="#d97706" />
          </svg>
          Vigil
        </Link>
        <div className="text-sm text-[var(--vigil-dim)]">
          Operational Incident Intelligence · Local-first · Replay-native · Zero-cost core
        </div>
      </div>
    </footer>
  )
}

/* ─── MAIN LANDING PAGE ─── */
export default function Landing() {
  const { theme, toggle } = useTheme()

  return (
    <main className="overflow-x-hidden w-full max-w-full">
      <Nav theme={theme} onToggle={toggle} />
      <Hero />
      <PulseBar />
      <Features />
      <Marquee />
      <Workflow />
      <Accordions />
      <Integrity />
      <Cta />
      <Footer />
    </main>
  )
}
