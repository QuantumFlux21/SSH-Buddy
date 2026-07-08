use std::{
    net::{TcpStream, ToSocketAddrs},
    process::Command,
    time::{Duration, Instant},
};

use crate::{
    domain::{
        now_timestamp, AppResult, PingCheck, PortScanReport, PortScanResult, RdpSettings,
        ServerProfile, ServerStatus, TcpCheck, SERVER_STATUS_DEGRADED, SERVER_STATUS_OFFLINE,
        SERVER_STATUS_ONLINE,
    },
    launcher::command_in_path,
};

const PING_COUNT: &str = "4";
const PING_TIMEOUT_SECONDS: &str = "2";
const TCP_TIMEOUT: Duration = Duration::from_millis(900);
const SCAN_TIMEOUT: Duration = Duration::from_millis(650);
const DEFAULT_SCAN_PORTS: &[(u16, &str)] = &[
    (22, "SSH"),
    (80, "HTTP"),
    (443, "HTTPS"),
    (3389, "RDP"),
    (5432, "PostgreSQL"),
    (6379, "Redis"),
    (8080, "HTTP-alt"),
    (8443, "HTTPS-alt"),
];

pub fn check_server_status(
    server: &ServerProfile,
    rdp_settings: Option<&RdpSettings>,
) -> AppResult<ServerStatus> {
    let host = normalized_host(server)?;
    let (primary_port, primary_service) = primary_service(server, rdp_settings);
    let ping = ping_host(&host);
    let tcp = tcp_connect(&host, primary_port, TCP_TIMEOUT);
    let ping_success = if ping.available && ping.attempted {
        Some(ping.success)
    } else {
        None
    };
    let state = status_state_for(ping_success, tcp.success);

    Ok(ServerStatus {
        server_id: server.id.clone(),
        state,
        checked_at: now_timestamp(),
        host,
        primary_port,
        primary_service,
        ping,
        tcp,
    })
}

pub fn scan_server_ports(server: &ServerProfile) -> AppResult<PortScanReport> {
    scan_server_ports_with(server, DEFAULT_SCAN_PORTS, |host, port| {
        tcp_connect(host, port, SCAN_TIMEOUT)
    })
}

pub fn scan_server_ports_with<F>(
    server: &ServerProfile,
    ports: &[(u16, &str)],
    mut connect: F,
) -> AppResult<PortScanReport>
where
    F: FnMut(&str, u16) -> TcpCheck,
{
    let host = normalized_host(server)?;
    let results = ports
        .iter()
        .map(|(port, label)| {
            let check = connect(&host, *port);
            PortScanResult {
                port: *port,
                label: (*label).to_string(),
                state: port_scan_state(&check).to_string(),
                latency_ms: check.latency_ms,
                error: check.error,
            }
        })
        .collect();

    Ok(PortScanReport {
        server_id: server.id.clone(),
        host,
        scanned_at: now_timestamp(),
        results,
        warning: "Manual selected-host scan only. Use this only on servers you own or administer."
            .to_string(),
    })
}

pub fn status_state_for(ping_success: Option<bool>, tcp_success: bool) -> String {
    match (ping_success, tcp_success) {
        (Some(true), true) => SERVER_STATUS_ONLINE.to_string(),
        (Some(true), false) | (Some(false), true) | (None, true) => {
            SERVER_STATUS_DEGRADED.to_string()
        }
        _ => SERVER_STATUS_OFFLINE.to_string(),
    }
}

pub fn parse_ping_output(status_success: bool, stdout: &str, stderr: &str) -> PingCheck {
    let combined = [stdout, stderr].join("\n");
    let packet_loss_percent = parse_packet_loss(&combined);
    let (min_ms, avg_ms, max_ms, mdev_ms) = parse_rtt_stats(&combined);
    let success = status_success && packet_loss_percent.map(|loss| loss < 100.0).unwrap_or(true);
    let error = if success {
        None
    } else {
        let trimmed = stderr.trim();
        if trimmed.is_empty() {
            Some("ping did not receive a successful response".to_string())
        } else {
            Some(trimmed.to_string())
        }
    };

    PingCheck {
        attempted: true,
        available: true,
        success,
        packet_loss_percent,
        min_ms,
        avg_ms,
        max_ms,
        mdev_ms,
        error,
    }
}

fn ping_host(host: &str) -> PingCheck {
    if !command_in_path("ping") {
        return PingCheck {
            attempted: false,
            available: false,
            success: false,
            packet_loss_percent: None,
            min_ms: None,
            avg_ms: None,
            max_ms: None,
            mdev_ms: None,
            error: Some(
                "ping command was not found; TCP reachability was still checked".to_string(),
            ),
        };
    }

    match Command::new("ping")
        .args(["-c", PING_COUNT, "-W", PING_TIMEOUT_SECONDS, "--", host])
        .output()
    {
        Ok(output) => parse_ping_output(
            output.status.success(),
            &String::from_utf8_lossy(&output.stdout),
            &String::from_utf8_lossy(&output.stderr),
        ),
        Err(error) => PingCheck {
            attempted: true,
            available: true,
            success: false,
            packet_loss_percent: None,
            min_ms: None,
            avg_ms: None,
            max_ms: None,
            mdev_ms: None,
            error: Some(format!("Failed to run ping: {error}")),
        },
    }
}

fn normalized_host(server: &ServerProfile) -> AppResult<String> {
    let host = server.host.trim();
    if host.is_empty() {
        return Err("Server profile is missing a host".to_string());
    }

    if host.starts_with('-') || host.chars().any(char::is_whitespace) {
        return Err("Host is not valid for reachability checks".to_string());
    }

    Ok(host.to_string())
}

fn tcp_connect(host: &str, port: u16, timeout: Duration) -> TcpCheck {
    let attempted = true;
    let addrs = match (host, port).to_socket_addrs() {
        Ok(addrs) => addrs.collect::<Vec<_>>(),
        Err(error) => {
            return TcpCheck {
                attempted,
                success: false,
                latency_ms: None,
                error: Some(format!("Failed to resolve host: {error}")),
            };
        }
    };

    if addrs.is_empty() {
        return TcpCheck {
            attempted,
            success: false,
            latency_ms: None,
            error: Some("Host resolved to no socket addresses".to_string()),
        };
    }

    let mut last_error = None;
    for addr in addrs {
        let started = Instant::now();
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(_) => {
                return TcpCheck {
                    attempted,
                    success: true,
                    latency_ms: Some(started.elapsed().as_secs_f64() * 1000.0),
                    error: None,
                };
            }
            Err(error) => {
                last_error = Some(if error.kind() == std::io::ErrorKind::TimedOut {
                    format!("Timed out connecting to {addr}")
                } else {
                    error.to_string()
                });
            }
        }
    }

    TcpCheck {
        attempted,
        success: false,
        latency_ms: None,
        error: last_error.or_else(|| Some("TCP connect failed".to_string())),
    }
}

fn port_scan_state(check: &TcpCheck) -> &'static str {
    if check.success {
        return "open";
    }

    let Some(error) = check.error.as_deref() else {
        return "closed";
    };
    let normalized = error.to_lowercase();

    if normalized.contains("timed out") {
        "timeout"
    } else if normalized.contains("connection refused") {
        "closed"
    } else {
        "error"
    }
}

fn primary_service(server: &ServerProfile, rdp_settings: Option<&RdpSettings>) -> (u16, String) {
    if let Some(settings) = rdp_settings {
        if settings.enabled {
            return (settings.port, "rdp".to_string());
        }
    }

    (server.port, "ssh".to_string())
}

fn parse_packet_loss(output: &str) -> Option<f64> {
    output.split(',').find_map(|segment| {
        let segment = segment.trim();
        segment
            .strip_suffix("% packet loss")
            .and_then(|value| value.trim().parse::<f64>().ok())
    })
}

fn parse_rtt_stats(output: &str) -> (Option<f64>, Option<f64>, Option<f64>, Option<f64>) {
    for line in output.lines() {
        if !line.contains("min/avg/max") {
            continue;
        }

        let Some((_, values)) = line.split_once('=') else {
            continue;
        };
        let Some(values) = values.split_whitespace().next() else {
            continue;
        };
        let parsed = values
            .split('/')
            .filter_map(|value| value.parse::<f64>().ok())
            .collect::<Vec<_>>();
        if parsed.len() >= 3 {
            return (
                parsed.first().copied(),
                parsed.get(1).copied(),
                parsed.get(2).copied(),
                parsed.get(3).copied(),
            );
        }
    }

    (None, None, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Tag, RDP_CERTIFICATE_MODE_TOFU, RDP_SCALING_MODE_NATIVE};
    use std::net::TcpListener;

    fn sample_server() -> ServerProfile {
        ServerProfile {
            id: "srv".to_string(),
            display_name: "NAS".to_string(),
            host: "127.0.0.1".to_string(),
            port: 22,
            username: "admin".to_string(),
            identity_file_id: None,
            proxy_jump: None,
            group_id: None,
            notes: None,
            favorite: false,
            tags: Vec::<Tag>::new(),
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        }
    }

    fn sample_rdp_settings() -> RdpSettings {
        RdpSettings {
            server_profile_id: "srv".to_string(),
            enabled: true,
            username: None,
            domain: None,
            port: 3389,
            certificate_mode: RDP_CERTIFICATE_MODE_TOFU.to_string(),
            fullscreen: false,
            multi_monitor: false,
            monitor_ids: None,
            width: None,
            height: None,
            color_depth: None,
            scaling_mode: RDP_SCALING_MODE_NATIVE.to_string(),
            scaling_percent: None,
            created_at: "2026-01-01T00:00:00.000Z".to_string(),
            updated_at: "2026-01-01T00:00:00.000Z".to_string(),
        }
    }

    #[test]
    fn maps_server_status_states() {
        assert_eq!(status_state_for(Some(true), true), "online");
        assert_eq!(status_state_for(Some(true), false), "degraded");
        assert_eq!(status_state_for(Some(false), true), "degraded");
        assert_eq!(status_state_for(Some(false), false), "offline");
        assert_eq!(status_state_for(None, true), "degraded");
        assert_eq!(status_state_for(None, false), "offline");
    }

    #[test]
    fn parses_linux_ping_output_with_mdev() {
        let output = "4 packets transmitted, 4 received, 0% packet loss, time 3006ms\nrtt min/avg/max/mdev = 17.991/18.071/18.127/0.043 ms";
        let ping = parse_ping_output(true, output, "");

        assert!(ping.success);
        assert_eq!(ping.packet_loss_percent, Some(0.0));
        assert_eq!(ping.min_ms, Some(17.991));
        assert_eq!(ping.avg_ms, Some(18.071));
        assert_eq!(ping.max_ms, Some(18.127));
        assert_eq!(ping.mdev_ms, Some(0.043));
    }

    #[test]
    fn parses_ping_failure_loss() {
        let output = "4 packets transmitted, 0 received, 100% packet loss, time 3071ms";
        let ping = parse_ping_output(false, output, "");

        assert!(!ping.success);
        assert_eq!(ping.packet_loss_percent, Some(100.0));
        assert!(ping.error.is_some());
    }

    #[test]
    fn tcp_connect_reports_success_and_failure() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        let success = tcp_connect("127.0.0.1", port, Duration::from_millis(200));
        assert!(success.success);
        assert!(success.latency_ms.is_some());

        drop(listener);
        let failure = tcp_connect("127.0.0.1", port, Duration::from_millis(50));
        assert!(!failure.success);
    }

    #[test]
    fn rdp_settings_override_primary_status_port() {
        let server = sample_server();
        let (port, service) = primary_service(&server, Some(&sample_rdp_settings()));

        assert_eq!(port, 3389);
        assert_eq!(service, "rdp");
    }

    #[test]
    fn selected_host_port_scan_uses_allowlist() {
        let report = scan_server_ports_with(&sample_server(), DEFAULT_SCAN_PORTS, |_host, port| {
            TcpCheck {
                attempted: true,
                success: port == 22,
                latency_ms: if port == 22 { Some(3.0) } else { None },
                error: if port == 22 {
                    None
                } else {
                    Some("connection refused".to_string())
                },
            }
        })
        .unwrap();

        assert_eq!(report.results.len(), 8);
        assert_eq!(report.results[0].port, 22);
        assert_eq!(report.results[0].state, "open");
        assert_eq!(report.results[1].state, "closed");
        assert!(report.warning.contains("selected-host"));
    }
}
