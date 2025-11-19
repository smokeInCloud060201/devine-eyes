use anyhow::Result;
use eyes_devine_shared::HttpRequest;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use crate::docker_service::DockerService;

/// Network monitoring service that captures HTTP requests from network traffic
pub struct NetworkMonitorService {
    docker_service: Arc<DockerService>,
    /// In-memory store of captured HTTP requests per container
    /// Key: container_id, Value: Vec<HttpRequest>
    captured_requests: Arc<RwLock<HashMap<String, Vec<HttpRequest>>>>,
}

impl NetworkMonitorService {
    pub fn new(docker_service: Arc<DockerService>) -> Self {
        Self {
            docker_service,
            captured_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get captured HTTP requests for a container
    pub async fn get_container_requests(&self, container_id: &str) -> Result<Vec<HttpRequest>> {
        let requests = self.captured_requests.read().await;
        Ok(requests
            .get(container_id)
            .cloned()
            .unwrap_or_default())
    }

    /// Start monitoring network traffic for HTTP requests
    /// This attempts to capture packets from Docker network interfaces
    pub async fn start_monitoring(&self) -> Result<()> {
        log::info!("Starting network-level HTTP request monitoring");
        
        // Get Docker network interfaces
        let network_interfaces = self.get_docker_network_interfaces().await?;
        
        if network_interfaces.is_empty() {
            log::warn!("No Docker network interfaces found. Network monitoring may not work.");
            log::warn!("Note: Network packet capture requires root privileges or CAP_NET_RAW capability.");
            return Ok(());
        }

        log::info!("Found {} Docker network interface(s)", network_interfaces.len());
        
        // For each interface, attempt to start packet capture
        for interface in network_interfaces {
            log::info!("Attempting to monitor interface: {}", interface);
            
            // Start packet capture in background task
            let monitor_clone = Arc::new(NetworkMonitorService {
                docker_service: self.docker_service.clone(),
                captured_requests: Arc::clone(&self.captured_requests),
            });
            
            let interface_clone = interface.clone();
            tokio::spawn(async move {
                if let Err(e) = monitor_clone.capture_packets(&interface_clone).await {
                    log::warn!("Failed to capture packets on {}: {}", interface_clone, e);
                }
            });
        }

        Ok(())
    }

    /// Get Docker network interfaces (bridge networks)
    async fn get_docker_network_interfaces(&self) -> Result<Vec<String>> {
        // Common Docker network interface names
        let _possible_interfaces = vec![
            "docker0",           // Default Docker bridge
            "br-",               // Custom bridge networks (prefix)
            "veth",              // Virtual ethernet pairs (prefix)
        ];

        // Try to detect Docker network interfaces
        // On Linux, we can check /sys/class/net
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            let net_dir = "/sys/class/net";
            let mut interfaces = Vec::new();
            
            if let Ok(entries) = fs::read_dir(net_dir) {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if possible_interfaces.iter().any(|prefix| name.starts_with(prefix)) {
                            interfaces.push(name);
                        }
                    }
                }
            }
            
            return Ok(interfaces);
        }

        // Windows: Docker Desktop uses WSL2, interfaces are in the VM
        #[cfg(target_os = "windows")]
        {
            // On Windows, Docker Desktop runs in WSL2
            // We can't directly access Docker network interfaces from Windows
            // Options:
            // 1. Use log parsing (works on Windows)
            // 2. Run the server inside WSL2
            // 3. Use Docker Desktop's network inspection
            log::info!("Windows detected. Network packet capture requires WSL2 or running inside Docker.");
            log::info!("Falling back to log parsing, which works on Windows.");
            
            // Try to find Docker Desktop network adapter (if using Hyper-V)
            // This is a best-effort attempt
            let mut interfaces = Vec::new();
            
            // Check for Docker Desktop network adapters
            // These are typically named like "vEthernet (WSL)" or "DockerNAT"
            if let Ok(output) = std::process::Command::new("powershell")
                .args(&["-Command", "Get-NetAdapter | Where-Object {$_.Name -like '*Docker*' -or $_.Name -like '*WSL*'} | Select-Object -ExpandProperty Name"])
                .output()
            {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    for line in output_str.lines() {
                        let name = line.trim();
                        if !name.is_empty() {
                            interfaces.push(name.to_string());
                        }
                    }
                }
            }
            
            Ok(interfaces)
        }
        
        // macOS: Similar to Windows, Docker Desktop uses a VM
        #[cfg(target_os = "macos")]
        {
            log::info!("macOS detected. Docker Desktop uses a VM, network capture may be limited.");
            log::info!("Falling back to log parsing, which works on macOS.");
            Ok(vec![])
        }
    }

    /// Parse HTTP request/response from packet data
    /// Uses httparse crate (no need to implement HTTP parser from scratch)
    fn parse_http_from_packet(&self, packet_data: &[u8]) -> Option<ParsedHttpRequest> {
        use httparse::{Request, Response, Status};
        
        // Try parsing as HTTP request first
        let mut req_headers = [httparse::EMPTY_HEADER; 64];
        let mut req = Request::new(&mut req_headers);
        
        match req.parse(packet_data) {
            Ok(Status::Complete(_)) => {
                // Successfully parsed as HTTP request
                return Some(ParsedHttpRequest {
                    method: req.method?.to_string(),
                    path: req.path?.to_string(),
                    status: None, // This is a request, not a response
                    response_time_ms: None,
                });
            }
            Ok(Status::Partial) => {
                // Partial request, might need more data (TCP reassembly needed)
                // For now, skip
            }
            Err(_) => {
                // Not a valid HTTP request, try parsing as response
            }
        }
        
        // Try parsing as HTTP response
        let mut resp_headers = [httparse::EMPTY_HEADER; 64];
        let mut resp = Response::new(&mut resp_headers);
        
        match resp.parse(packet_data) {
            Ok(Status::Complete(_)) => {
                // Successfully parsed as HTTP response
                return Some(ParsedHttpRequest {
                    method: "RESPONSE".to_string(), // We'll need to match with request
                    path: "/".to_string(),
                    status: resp.code,
                    response_time_ms: None, // Calculate from request timestamp
                });
            }
            Ok(Status::Partial) => {
                // Partial response, might need more data
            }
            Err(_) => {
                // Not a valid HTTP response
            }
        }
        
        None
    }

    /// Store a captured HTTP request
    pub async fn store_request(&self, container_id: String, request: HttpRequest) {
        let mut requests = self.captured_requests.write().await;
        let container_requests = requests.entry(container_id).or_insert_with(Vec::new);
        container_requests.push(request);
        
        // Keep only last 1000 requests per container to avoid memory issues
        if container_requests.len() > 1000 {
            container_requests.remove(0);
        }
    }

    /// Clear old requests (older than specified duration)
    pub async fn cleanup_old_requests(&self, max_age_seconds: i64) {
        let mut requests = self.captured_requests.write().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(max_age_seconds);
        
        for container_requests in requests.values_mut() {
            container_requests.retain(|req| {
                req.timestamp > cutoff
            });
        }
    }

    /// Capture packets from a network interface using pcap
    /// Uses existing pcap library - no need to implement packet capture from scratch
    async fn capture_packets(&self, _interface: &str) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use pcap::{Capture, Device};
            
            // Find the device
            let devices = Device::list()?;
            let device = devices
                .iter()
                .find(|d| d.name == interface)
                .ok_or_else(|| anyhow::anyhow!("Interface {} not found", interface))?;
            
            // Open capture (requires root or CAP_NET_RAW)
            let mut cap = match Capture::from_device(device.name.as_str()) {
                Ok(cap) => cap.promisc(true).snaplen(65535).open()?,
                Err(e) => {
                    log::warn!("Failed to open capture on {}: {}. Need root or CAP_NET_RAW", interface, e);
                    return Err(anyhow::anyhow!("Permission denied: {}", e));
                }
            };
            
            // Filter for HTTP traffic (ports 80, 8080, 8000, etc.)
            cap.filter("tcp port 80 or tcp port 8080 or tcp port 8000", true)?;
            
            log::info!("Started capturing packets on {}", interface);
            
            // Capture packets in a loop (blocking, so this runs in a separate task)
            loop {
                let packet = match cap.next_packet() {
                    Ok(p) => p,
                    Err(e) => {
                        log::error!("Error capturing packet: {}", e);
                        break;
                    }
                };
                // Extract TCP payload (skip IP/TCP headers)
                // This is simplified - real implementation needs TCP reassembly
                if let Some(http_data) = self.extract_http_payload(&packet.data) {
                    if let Some(parsed) = self.parse_http_from_packet(http_data) {
                        // Map packet to container by IP (simplified)
                        // Real implementation would track connections and map IPs to containers
                        if let Some(container_id) = self.find_container_for_packet(&packet).await {
                            let request = HttpRequest {
                                container_id: container_id.clone(),
                                container_name: "unknown".to_string(), // Will be filled from container info
                                endpoint: parsed.path,
                                method: parsed.method,
                                http_status: parsed.status.unwrap_or(200),
                                response_time_ms: parsed.response_time_ms.unwrap_or(0.0),
                                timestamp: Utc::now(),
                            };
                            self.store_request(container_id, request).await;
                        }
                    }
                }
            }
            
            Ok(())
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            log::warn!("Packet capture only supported on Linux");
            Err(anyhow::anyhow!("Packet capture not supported on this platform"))
        }
    }

    /// Extract HTTP payload from packet (simplified - real implementation needs TCP reassembly)
    fn extract_http_payload<'a>(&self, packet_data: &'a [u8]) -> Option<&'a [u8]> {
        // Skip Ethernet header (14 bytes) and IP header (variable, typically 20 bytes)
        // Skip TCP header (variable, typically 20 bytes)
        // This is simplified - real implementation needs proper IP/TCP parsing
        if packet_data.len() > 54 {
            Some(&packet_data[54..])
        } else {
            None
        }
    }

    /// Find container ID for a packet by matching IP addresses
    async fn find_container_for_packet(&self, _packet: &pcap::Packet<'_>) -> Option<String> {
        // TODO: Extract source/dest IP from packet
        // TODO: Match IP to container using Docker API
        // For now, return None - this needs proper IP extraction and container mapping
        None
    }
}

/// Parsed HTTP request from network packet
struct ParsedHttpRequest {
    method: String,
    path: String,
    status: Option<u16>,  // None for request, Some for response
    response_time_ms: Option<f64>,
}

