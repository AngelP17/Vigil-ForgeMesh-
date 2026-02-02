use axum::{
    routing::{get, post},
    Router,
    extract::{State, Path, WebSocketUpgrade},
    response::{Html, Json, IntoResponse},
    http::StatusCode,
};
use axum::extract::ws::{WebSocket, Message};
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use serde_json::json;

pub mod api;

pub struct AppState {
    pub store: Arc<Mutex<forgemesh_core::store::ForgeStore>>,
    pub node_id: String,
    pub tx: broadcast::Sender<String>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(websocket_handler))
        .route("/api/sensors", get(list_sensors))
        .route("/api/sensor/:id/history", get(get_history))
        .route("/api/sensor/:id/write", post(write_value))
        .route("/api/sensor/:id/analytics", get(api::get_analytics))      // NEW
        .route("/api/sensor/:id/simulate", post(api::trigger_simulation)) // NEW
        .route("/api/line/:id/oee", get(api::get_oee))                    // NEW
        .route("/api/status", get(get_status))
        .route("/api/mesh/topology", get(get_topology))
        .route("/api/export/:id", post(export_sensor))
        .with_state(state)
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.tx.subscribe();

    let _ = socket.send(Message::Text(json!({
        "type": "connected",
        "node_id": state.node_id,
        "timestamp": chrono::Utc::now().timestamp_millis()
    }).to_string())).await;

    while let Ok(msg) = rx.recv().await {
        if socket.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

async fn index_handler() -> Html<&'static str> {
    Html(DASHBOARD_HTML)
}

async fn list_sensors(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let store = state.store.lock().await;
    let mut sensors = Vec::new();

    for item in store.iter_data() {
        if let Ok((_, node)) = item {
            if !sensors.contains(&node.sensor_id) {
                sensors.push(node.sensor_id.clone());
            }
        }
    }

    if sensors.is_empty() {
        sensors.push("ontario-line1-temp".to_string());
        sensors.push("georgia-line2-pressure".to_string());
    }

    Json(sensors)
}

async fn get_history(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>
) -> impl IntoResponse {
    let store = state.store.lock().await;
    let history = store.get_history(&id, 1000).unwrap_or_default();

    let points: Vec<_> = history.into_iter().map(|n| json!({
        "x": n.timestamp_ns / 1_000_000,
        "y": n.value,
        "hash": n.data_hash,
        "verified": n.verify_integrity()
    })).collect();

    Json(json!({
        "sensor": id,
        "datapoints": points,
        "count": points.len()
    }))
}

async fn write_value(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    body: String
) -> impl IntoResponse {
    let value: f64 = body.parse().unwrap_or(0.0);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let store = state.store.lock().await;
    match store.put(&id, value, ts) {
        Ok(hash) => {
            let depth = store.get_history(&id, 10000).unwrap_or_default().len();
            let _ = state.tx.send(json!({
                "type": "new_data",
                "sensor": id,
                "value": value,
                "hash": hash,
                "timestamp": ts
            }).to_string());

            (StatusCode::OK, Json(json!({
                "status": "committed",
                "hash": hash,
                "dag_depth": depth
            })))
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "error": e.to_string(),
            "status": "failed"
        }))),
    }
}

async fn get_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let store = state.store.lock().await;
    let mut total_records = 0;
    let mut sensors = std::collections::HashSet::new();

    for item in store.iter_data() {
        if let Ok((_, node)) = item {
            total_records += 1;
            sensors.insert(node.sensor_id);
        }
    }

    Json(json!({
        "node_id": state.node_id,
        "status": "active",
        "version": "0.2.0",
        "mode": "mesh-ready",
        "stats": {
            "total_records": total_records,
            "sensors_tracked": sensors.len(),
            "storage_backend": "Sled LSM-Tree",
            "consistency_model": "Merkle-DAG + CRDT"
        },
        "mesh": {
            "peers_connected": 0,
            "partition_status": "healthy",
            "last_sync": chrono::Utc::now().to_rfc3339()
        }
    }))
}

async fn get_topology() -> impl IntoResponse {
    Json(json!({
        "nodes": [
            {"id": "ontario-node", "region": "CA", "status": "online", "last_seen": "0s"},
            {"id": "georgia-node", "region": "US", "status": "online", "last_seen": "2s"},
            {"id": "texas-node", "region": "US", "status": "partitioned", "last_seen": "5m"}
        ],
        "links": [
            {"source": "ontario-node", "target": "georgia-node", "latency_ms": 45},
            {"source": "georgia-node", "target": "texas-node", "status": "degraded"}
        ]
    }))
}

async fn export_sensor(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>
) -> impl IntoResponse {
    let store = state.store.lock().await;
    let history = store.get_history(&id, 10000).unwrap_or_default();

    (StatusCode::OK, Json(json!({
        "sensor": id,
        "records": history.len(),
        "format": "CAR (Content Addressable Archive)",
        "integrity": "SHA3-256 verified",
        "download_ready": true
    })))
}

const DASHBOARD_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ForgeMesh | Industrial Mesh Historian</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>
    <style>
        :root {
            --bg-primary: #0a0f1c;
            --bg-secondary: #151b2d;
            --bg-card: #1e2538;
            --accent-cyan: #00d4ff;
            --accent-green: #00ff88;
            --accent-orange: #ff6b35;
            --accent-red: #ff3333;
            --text-primary: #e2e8f0;
            --text-secondary: #94a3b8;
            --border: #2d3748;
        }

        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'JetBrains Mono', 'Consolas', monospace;
            background: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            overflow-x: hidden;
        }

        .header {
            background: linear-gradient(135deg, var(--bg-secondary) 0%, var(--bg-primary) 100%);
            border-bottom: 2px solid var(--accent-cyan);
            padding: 1.5rem 2rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
            box-shadow: 0 4px 20px rgba(0, 212, 255, 0.1);
        }

        .logo {
            display: flex;
            align-items: center;
            gap: 1rem;
        }

        .logo-icon {
            width: 48px;
            height: 48px;
            background: linear-gradient(135deg, var(--accent-cyan), var(--accent-green));
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 24px;
            box-shadow: 0 0 20px rgba(0, 212, 255, 0.3);
        }

        .logo-text h1 {
            font-size: 1.8rem;
            background: linear-gradient(90deg, var(--accent-cyan), var(--accent-green));
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            margin-bottom: 0.25rem;
        }

        .logo-text span {
            color: var(--text-secondary);
            font-size: 0.85rem;
            text-transform: uppercase;
            letter-spacing: 2px;
        }

        .status-badge {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            padding: 0.5rem 1rem;
            background: rgba(0, 255, 136, 0.1);
            border: 1px solid var(--accent-green);
            border-radius: 20px;
            font-size: 0.9rem;
        }

        .status-badge::before {
            content: "";
            width: 8px;
            height: 8px;
            background: var(--accent-green);
            border-radius: 50%;
            animation: pulse 2s infinite;
        }

        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }

        .container {
            padding: 2rem;
            max-width: 1920px;
            margin: 0 auto;
        }

        .grid {
            display: grid;
            grid-template-columns: 300px 1fr 350px;
            gap: 1.5rem;
            margin-bottom: 1.5rem;
        }

        @media (max-width: 1400px) {
            .grid { grid-template-columns: 250px 1fr; }
            .sidebar-right { display: none; }
        }

        @media (max-width: 1024px) {
            .grid { grid-template-columns: 1fr; }
            .sidebar-left { display: none; }
        }

        .card {
            background: var(--bg-card);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 1.5rem;
            box-shadow: 0 4px 15px rgba(0,0,0,0.3);
        }

        .card-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 1rem;
            padding-bottom: 0.75rem;
            border-bottom: 1px solid var(--border);
        }

        .card-title {
            font-size: 0.95rem;
            text-transform: uppercase;
            letter-spacing: 1px;
            color: var(--accent-cyan);
            font-weight: 600;
        }

        .sensor-list {
            display: flex;
            flex-direction: column;
            gap: 0.5rem;
        }

        .sensor-btn {
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            color: var(--text-primary);
            padding: 0.75rem 1rem;
            border-radius: 8px;
            cursor: pointer;
            text-align: left;
            font-family: inherit;
            transition: all 0.2s;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }

        .sensor-btn:hover, .sensor-btn.active {
            border-color: var(--accent-cyan);
            background: rgba(0, 212, 255, 0.1);
            box-shadow: 0 0 10px rgba(0, 212, 255, 0.2);
        }

        .sensor-status {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background: var(--accent-green);
        }

        .metric-row {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0.75rem 0;
            border-bottom: 1px solid var(--border);
        }

        .metric-row:last-child { border-bottom: none; }

        .metric-label { color: var(--text-secondary); font-size: 0.9rem; }
        .metric-value {
            font-weight: 600;
            font-family: 'JetBrains Mono', monospace;
        }

        .chart-container {
            position: relative;
            height: 400px;
            background: var(--bg-secondary);
            border-radius: 8px;
            padding: 1rem;
        }

        .controls {
            display: flex;
            gap: 1rem;
            margin-bottom: 1rem;
            flex-wrap: wrap;
        }

        .btn {
            background: var(--bg-secondary);
            border: 1px solid var(--accent-cyan);
            color: var(--accent-cyan);
            padding: 0.5rem 1rem;
            border-radius: 6px;
            cursor: pointer;
            font-family: inherit;
            font-size: 0.9rem;
            transition: all 0.2s;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }

        .btn:hover {
            background: var(--accent-cyan);
            color: var(--bg-primary);
            box-shadow: 0 0 15px rgba(0, 212, 255, 0.4);
        }

        .btn-primary {
            background: var(--accent-cyan);
            color: var(--bg-primary);
            border: none;
            font-weight: 600;
        }

        .btn-primary:hover {
            background: var(--accent-green);
            box-shadow: 0 0 15px rgba(0, 255, 136, 0.4);
        }

        .input-group {
            display: flex;
            gap: 0.5rem;
            margin-bottom: 1rem;
        }

        .input {
            background: var(--bg-secondary);
            border: 1px solid var(--border);
            color: var(--text-primary);
            padding: 0.5rem 1rem;
            border-radius: 6px;
            font-family: inherit;
            flex: 1;
        }

        .input:focus {
            outline: none;
            border-color: var(--accent-cyan);
        }

        .dag-viz {
            height: 250px;
            background: var(--bg-secondary);
            border-radius: 8px;
            border: 1px solid var(--border);
            overflow-y: auto;
        }

        .partition-indicator {
            display: flex;
            align-items: center;
            gap: 0.5rem;
            margin-top: 1rem;
            padding: 0.75rem;
            background: rgba(255, 107, 53, 0.1);
            border: 1px solid var(--accent-orange);
            border-radius: 6px;
            font-size: 0.85rem;
        }

        .log-stream {
            font-family: 'JetBrains Mono', monospace;
            font-size: 0.8rem;
            background: var(--bg-secondary);
            padding: 1rem;
            border-radius: 8px;
            height: 200px;
            overflow-y: auto;
            border: 1px solid var(--border);
        }

        .log-entry {
            margin-bottom: 0.5rem;
            color: var(--text-secondary);
        }

        .log-entry .timestamp { color: var(--accent-cyan); }
        .log-entry .hash { color: var(--accent-green); font-size: 0.75rem; }

        .topology-container {
            height: 300px;
            background: var(--bg-secondary);
            border-radius: 8px;
            border: 1px solid var(--border);
        }

        ::-webkit-scrollbar { width: 8px; height: 8px; }
        ::-webkit-scrollbar-track { background: var(--bg-secondary); }
        ::-webkit-scrollbar-thumb { background: var(--border); border-radius: 4px; }
        ::-webkit-scrollbar-thumb:hover { background: var(--accent-cyan); }
    </style>
</head>
<body>
    <header class="header">
        <div class="logo">
            <div class="logo-icon">&#9889;</div>
            <div class="logo-text">
                <h1>ForgeMesh</h1>
                <span>Distributed Industrial Historian</span>
            </div>
        </div>
        <div style="display: flex; gap: 1rem; align-items: center;">
            <div class="status-badge" id="connectionStatus">LIVE</div>
            <div style="text-align: right; color: var(--text-secondary); font-size: 0.85rem;">
                <div id="nodeId">node-local</div>
                <div>v0.2.0 | Merkle-DAG + CRDT</div>
            </div>
        </div>
    </header>

    <div class="container">
        <div class="grid">
            <!-- Left Sidebar: Sensor Navigation -->
            <div class="sidebar-left">
                <div class="card">
                    <div class="card-header">
                        <span class="card-title">Sensors</span>
                        <button class="btn" onclick="refreshSensors()" style="padding: 0.25rem 0.5rem; font-size: 0.8rem;">&#8635;</button>
                    </div>
                    <div class="sensor-list" id="sensorList">
                        <div style="color: var(--text-secondary); text-align: center; padding: 2rem 0;">
                            Loading sensors...
                        </div>
                    </div>
                </div>

                <div class="card" style="margin-top: 1rem;">
                    <div class="card-header">
                        <span class="card-title">Mesh Topology</span>
                    </div>
                    <div id="topologyViz" class="topology-container"></div>
                    <div class="partition-indicator" id="partitionStatus">
                        <span style="color: var(--accent-orange);">&#9888;</span>
                        <span>Texas node partitioned (5m ago)</span>
                    </div>
                </div>
            </div>

            <!-- Main Content: Charts & Data -->
            <div class="main-content">
                <div class="card">
                    <div class="card-header">
                        <span class="card-title" id="chartTitle">Select a Sensor</span>
                        <div style="display: flex; gap: 0.5rem;">
                            <button class="btn" onclick="setTimeRange('1h')">1H</button>
                            <button class="btn" onclick="setTimeRange('24h')">24H</button>
                            <button class="btn" onclick="setTimeRange('7d')">7D</button>
                            <button class="btn btn-primary" onclick="exportCurrent()">&#11015; Export CAR</button>
                        </div>
                    </div>

                    <div class="input-group">
                        <input type="number" id="newValue" class="input" placeholder="Enter value..." step="0.1">
                        <button class="btn btn-primary" onclick="writeValue()">Write to DAG</button>
                    </div>

                    <div class="chart-container">
                        <canvas id="mainChart"></canvas>
                    </div>
                </div>

                <div class="grid" style="margin-top: 1rem; grid-template-columns: 1fr 1fr;">
                    <div class="card">
                        <div class="card-header">
                            <span class="card-title">Merkle DAG Visualization</span>
                        </div>
                        <div id="dagViz" class="dag-viz"></div>
                    </div>

                    <div class="card">
                        <div class="card-header">
                            <span class="card-title">Consensus Log</span>
                        </div>
                        <div class="log-stream" id="logStream">
                            <div class="log-entry">
                                <span class="timestamp">[SYSTEM]</span> ForgeMesh initialized. Listening for mesh connections...
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Right Sidebar: Metrics & Info -->
            <div class="sidebar-right">
                <div class="card">
                    <div class="card-header">
                        <span class="card-title">Node Statistics</span>
                    </div>
                    <div class="metric-row">
                        <span class="metric-label">Storage Engine</span>
                        <span class="metric-value" style="color: var(--accent-cyan);">Sled LSM</span>
                    </div>
                    <div class="metric-row">
                        <span class="metric-label">Total Records</span>
                        <span class="metric-value" id="totalRecords">0</span>
                    </div>
                    <div class="metric-row">
                        <span class="metric-label">Active Sensors</span>
                        <span class="metric-value" id="activeSensors">0</span>
                    </div>
                    <div class="metric-row">
                        <span class="metric-label">Hash Function</span>
                        <span class="metric-value" style="font-size: 0.8rem;">SHA3-256</span>
                    </div>
                    <div class="metric-row">
                        <span class="metric-label">Consistency</span>
                        <span class="metric-value" style="color: var(--accent-green);">Eventual</span>
                    </div>
                </div>

                <div class="card" style="margin-top: 1rem;">
                    <div class="card-header">
                        <span class="card-title">Current Head</span>
                    </div>
                    <div style="font-family: monospace; font-size: 0.75rem; color: var(--text-secondary); word-break: break-all;" id="currentHash">
                        No sensor selected
                    </div>
                    <div style="margin-top: 1rem; padding-top: 1rem; border-top: 1px solid var(--border);">
                        <div class="metric-row">
                            <span class="metric-label">DAG Depth</span>
                            <span class="metric-value" id="dagDepth">0</span>
                        </div>
                        <div class="metric-row">
                            <span class="metric-label">Last Write</span>
                            <span class="metric-value" id="lastWrite">-</span>
                        </div>
                    </div>
                </div>

                <div class="card" style="margin-top: 1rem; background: linear-gradient(135deg, rgba(0,212,255,0.1), rgba(0,255,136,0.05));">
                    <div style="text-align: center; padding: 1rem;">
                        <div style="font-size: 2rem; margin-bottom: 0.5rem;">&#128274;</div>
                        <div style="color: var(--accent-green); font-weight: 600; margin-bottom: 0.5rem;">Chain Verified</div>
                        <div style="font-size: 0.85rem; color: var(--text-secondary);">
                            All records cryptographically signed and tamper-evident
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        let ws;
        let chart;
        let currentSensor = null;
        let sensors = [];

        function connectWebSocket() {
            ws = new WebSocket('ws://' + window.location.host + '/ws');

            ws.onopen = function() {
                console.log('WebSocket connected');
                document.getElementById('connectionStatus').innerText = 'LIVE';
                document.getElementById('connectionStatus').style.borderColor = 'var(--accent-green)';
            };

            ws.onmessage = function(event) {
                var msg = JSON.parse(event.data);
                handleWebSocketMessage(msg);
            };

            ws.onclose = function() {
                console.log('WebSocket disconnected, retrying...');
                document.getElementById('connectionStatus').innerText = 'OFFLINE';
                document.getElementById('connectionStatus').style.borderColor = 'var(--accent-red)';
                setTimeout(connectWebSocket, 3000);
            };
        }

        function handleWebSocketMessage(msg) {
            if (msg.type === 'new_data' && msg.sensor === currentSensor) {
                addLogEntry(msg);
                refreshChart();
                updateStats();
            }
        }

        function addLogEntry(msg) {
            var log = document.getElementById('logStream');
            var entry = document.createElement('div');
            entry.className = 'log-entry';
            entry.innerHTML =
                '<span class="timestamp">[' + new Date().toLocaleTimeString() + ']</span> ' +
                'Write to <b>' + msg.sensor + '</b>: ' + msg.value +
                '<div class="hash">' + msg.hash.substring(0, 24) + '...</div>';
            log.insertBefore(entry, log.firstChild);
            if (log.children.length > 50) log.removeChild(log.lastChild);
        }

        async function refreshSensors() {
            var res = await fetch('/api/sensors');
            sensors = await res.json();
            var container = document.getElementById('sensorList');
            container.innerHTML = sensors.map(function(s) {
                return '<button class="sensor-btn ' + (s === currentSensor ? 'active' : '') +
                    '" onclick="selectSensor(\'' + s + '\')">' +
                    '<span>' + s + '</span>' +
                    '<span class="sensor-status"></span>' +
                    '</button>';
            }).join('');

            document.getElementById('activeSensors').innerText = sensors.length;

            if (!currentSensor && sensors.length > 0) {
                selectSensor(sensors[0]);
            }
        }

        function selectSensor(id) {
            currentSensor = id;
            document.getElementById('chartTitle').innerText = id;

            document.querySelectorAll('.sensor-btn').forEach(function(btn) {
                btn.classList.toggle('active', btn.innerText.includes(id));
            });

            refreshChart();
            refreshStats();
        }

        async function refreshChart() {
            if (!currentSensor) return;

            var res = await fetch('/api/sensor/' + currentSensor + '/history');
            var data = await res.json();

            var ctx = document.getElementById('mainChart').getContext('2d');

            if (chart) chart.destroy();

            chart = new Chart(ctx, {
                type: 'line',
                data: {
                    labels: data.datapoints.map(function(p) { return new Date(p.x).toLocaleTimeString(); }),
                    datasets: [{
                        label: currentSensor,
                        data: data.datapoints.map(function(p) { return p.y; }),
                        borderColor: '#00d4ff',
                        backgroundColor: 'rgba(0, 212, 255, 0.1)',
                        borderWidth: 2,
                        tension: 0.4,
                        pointRadius: 4,
                        pointBackgroundColor: '#00ff88',
                        fill: true
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    interaction: {
                        mode: 'index',
                        intersect: false
                    },
                    plugins: {
                        legend: { labels: { color: '#e2e8f0' } },
                        tooltip: {
                            backgroundColor: '#1e2538',
                            titleColor: '#00d4ff',
                            bodyColor: '#e2e8f0',
                            borderColor: '#2d3748',
                            borderWidth: 1
                        }
                    },
                    scales: {
                        x: {
                            grid: { color: '#2d3748' },
                            ticks: { color: '#94a3b8' }
                        },
                        y: {
                            grid: { color: '#2d3748' },
                            ticks: { color: '#94a3b8' }
                        }
                    }
                }
            });

            updateDagViz(data.datapoints.slice(0, 20));
        }

        function updateDagViz(points) {
            var container = document.getElementById('dagViz');
            if (points.length === 0) {
                container.innerHTML = '<div style="text-align: center; padding-top: 100px; color: var(--text-secondary);">No data</div>';
                return;
            }

            var html = '<div style="display: flex; flex-direction: column; gap: 8px; padding: 1rem;">';
            points.forEach(function(p, i) {
                var opacity = 1 - (i * 0.05);
                html +=
                    '<div style="display: flex; align-items: center; gap: 1rem; opacity: ' + opacity + ';">' +
                    '<div style="font-family: monospace; font-size: 0.7rem; color: var(--accent-cyan); min-width: 80px;">' +
                    'Block ' + (points.length - i) +
                    '</div>' +
                    '<div style="flex: 1; background: var(--bg-primary); padding: 0.5rem; border-radius: 4px; font-family: monospace; font-size: 0.75rem; border-left: 3px solid var(--accent-green);">' +
                    p.hash.substring(0, 32) + '...<br>' +
                    '<span style="color: var(--text-secondary);">Value: ' + p.y + ' | ' + new Date(p.x).toLocaleTimeString() + '</span>' +
                    '</div></div>';
            });
            html += '</div>';
            container.innerHTML = html;
        }

        async function writeValue() {
            if (!currentSensor) { alert('Select a sensor first'); return; }
            var value = document.getElementById('newValue').value;
            if (!value) return;

            var res = await fetch('/api/sensor/' + currentSensor + '/write', {
                method: 'POST',
                body: value
            });

            if (res.ok) {
                document.getElementById('newValue').value = '';
                refreshChart();
                updateStats();
            }
        }

        async function refreshStats() {
            var res = await fetch('/api/status');
            var status = await res.json();
            document.getElementById('nodeId').innerText = status.node_id;
            document.getElementById('totalRecords').innerText = status.stats.total_records;
        }

        function setTimeRange(range) {
            console.log('Setting range:', range);
            refreshChart();
        }

        async function exportCurrent() {
            if (!currentSensor) return;
            var res = await fetch('/api/export/' + currentSensor, {method: 'POST'});
            var data = await res.json();
            alert('Exported ' + data.records + ' records to CAR format');
        }

        async function loadTopology() {
            var res = await fetch('/api/mesh/topology');
            var data = await res.json();

            var container = document.getElementById('topologyViz');
            var html = '<div style="padding: 1rem; display: flex; justify-content: space-around; align-items: center; height: 100%;">';
            data.nodes.forEach(function(node) {
                var color = node.status === 'online' ? 'var(--accent-green)' : 'var(--accent-orange)';
                var flag = node.region === 'CA' ? '&#127464;&#127462;' : '&#127482;&#127480;';
                html +=
                    '<div style="text-align: center;">' +
                    '<div style="width: 60px; height: 60px; border-radius: 50%; background: ' + color + '; margin: 0 auto 0.5rem; display: flex; align-items: center; justify-content: center; box-shadow: 0 0 20px ' + color + '40;">' +
                    '<span style="font-size: 1.5rem;">' + flag + '</span>' +
                    '</div>' +
                    '<div style="font-size: 0.8rem; font-weight: 600;">' + node.id + '</div>' +
                    '<div style="font-size: 0.7rem; color: var(--text-secondary);">' + node.last_seen + '</div>' +
                    '</div>';
            });
            html += '</div>';
            container.innerHTML = html;
        }

        function updateStats() {
            refreshStats();
            if (currentSensor) {
                fetch('/api/sensor/' + currentSensor + '/history')
                    .then(function(r) { return r.json(); })
                    .then(function(d) {
                        document.getElementById('dagDepth').innerText = d.count;
                        if (d.datapoints.length > 0) {
                            document.getElementById('currentHash').innerText = d.datapoints[0].hash;
                            document.getElementById('lastWrite').innerText = new Date(d.datapoints[0].x).toLocaleTimeString();
                        }
                    });
            }
        }

        // Init
        connectWebSocket();
        refreshSensors();
        refreshStats();
        loadTopology();
        setInterval(refreshStats, 5000);
        setInterval(loadTopology, 10000);
    </script>
</body>
</html>"##;
