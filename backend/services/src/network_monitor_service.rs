use anyhow::Result;
use eyes_devine_shared::HttpRequest;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{Utc, DateTime, FixedOffset};
use sea_orm::{DatabaseConnection, ActiveValue::Set, ActiveModelTrait};
use crate::docker_service::DockerService;

/// Network monitoring service that captures HTTP requests from network traffic
pub struct NetworkMonitorService {
    docker_service: Arc<DockerService>,
    /// Optional database connection for direct insertion
    db: Option<Arc<DatabaseConnection>>,
    /// In-memory store of captured HTTP requests per container
    /// Key: container_id, Value: Vec<HttpRequest>
    captured_requests: Arc<RwLock<HashMap<String, Vec<HttpRequest>>>>,
    /// Track TCP connections to match requests with responses
    /// Key: connection_id (format: "src_ip:src_port-dst_ip:dst_port"), Value: PendingRequest
    pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
}

/// Pending HTTP request waiting for response
struct PendingRequest {
    container_id: String,
    container_name: String,
    method: String,
    endpoint: String,
    request_timestamp: DateTime<Utc>,
}

impl NetworkMonitorService {
    pub fn new(docker_service: Arc<DockerService>) -> Self {
        Self {
            docker_service,
            db: None,
            captured_requests: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new NetworkMonitorService with database connection for direct insertion
    pub fn with_database(docker_service: Arc<DockerService>, db: Arc<DatabaseConnection>) -> Self {
        Self {
            docker_service,
            db: Some(db),
            captured_requests: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get captured HTTP requests for a container
    pub async fn get_container_requests(&self, container_id: &str) -> Result<Vec<HttpRequest>> {
        #[cfg(not(feature = "network-capture"))]
        {
            log::debug!("Network capture feature not enabled - cannot get requests for container {}", container_id);
            return Err(anyhow::anyhow!("Network capture feature not enabled. Rebuild with --features network-capture"));
        }
        
        #[cfg(feature = "network-capture")]
        {
        let requests = self.captured_requests.read().await;
        Ok(requests
            .get(container_id)
            .cloned()
            .unwrap_or_default())
        }
    }

    /// Clear captured requests for a specific container
    pub async fn clear_container_requests(&self, container_id: &str) {
        let mut requests = self.captured_requests.write().await;
        requests.remove(container_id);
        log::debug!("Cleared captured requests for container {}", container_id);
    }

    /// Start monitoring network traffic for HTTP requests
    /// This attempts to capture packets from Docker network interfaces
    /// Works both when running locally (monitoring Docker containers) and when running in Docker
    pub async fn start_monitoring(&self) -> Result<()> {
        log::info!("Starting network-level HTTP request monitoring");
        
        // Detect if we're running inside Docker
        let is_running_in_docker = self.detect_if_in_docker().await;
        if is_running_in_docker {
            log::info!("Detected: Running inside Docker container - will monitor Docker network interfaces");
        } else {
            log::info!("Detected: Running on host - will monitor Docker bridge networks and host interfaces");
        }
        
        // Strategy: Capture from ALL networks
        // Since containers can be on multiple networks, we need to capture all traffic
        // When running in host network mode (Docker), use "any" interface to see all traffic
        // When running locally, capture on all Docker bridge interfaces + "any" interface
        
            #[cfg(target_os = "linux")]
            {
            // On Linux, use "any" interface to capture ALL network traffic
            // This includes all Docker networks, host traffic, and container-to-container traffic
            log::info!("Using 'any' interface to capture ALL network traffic (all Docker networks)");
                if let Err(e) = self.try_capture_on_interface("any").await {
                log::warn!("Failed to capture on 'any' interface: {}. Falling back to specific Docker interfaces.", e);
                
                // Fallback: Try to capture on specific Docker interfaces
                let network_interfaces = self.get_docker_network_interfaces().await?;
                if !network_interfaces.is_empty() {
                    log::info!("Found {} Docker network interface(s) as fallback", network_interfaces.len());
                    for interface in network_interfaces {
                        log::info!("Attempting to monitor interface: {}", interface);
                        let monitor_clone = Arc::new(NetworkMonitorService {
                            docker_service: self.docker_service.clone(),
                            db: self.db.clone(),
                            captured_requests: Arc::clone(&self.captured_requests),
                            pending_requests: Arc::clone(&self.pending_requests),
                        });
                        let interface_clone = interface.clone();
                        tokio::spawn(async move {
                            if let Err(e) = monitor_clone.capture_packets(&interface_clone).await {
                                log::warn!("Failed to capture packets on {}: {}", interface_clone, e);
                            }
                        });
                    }
                } else {
                    log::warn!("No Docker network interfaces found. Packet capture may not work.");
                }
            } else {
                log::info!("Successfully started capture on 'any' interface - monitoring ALL networks");
                }
            }
            
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
            // On macOS/Windows, try to get all Docker interfaces + use "any" if available
            let network_interfaces = self.get_docker_network_interfaces().await?;
            
            if network_interfaces.is_empty() {
                log::warn!("No Docker network interfaces found automatically.");
                log::info!("Attempting to use 'any' interface or default interface...");
                
                // Try "any" interface first (if available on this platform)
                let mut captured = false;
                if let Ok(()) = self.try_capture_on_interface("any").await {
                    log::info!("Successfully started capture on 'any' interface");
                    captured = true;
                }
                
                // If "any" didn't work, try common interfaces
                if !captured {
                let common_interfaces = if cfg!(target_os = "macos") {
                    vec!["en0", "en1", "bridge0"]
                } else {
                    vec!["Ethernet", "Wi-Fi", "Local Area Connection"]
                };
                
                for iface in common_interfaces {
                    if let Ok(()) = self.try_capture_on_interface(iface).await {
                        log::info!("Successfully started capture on {}", iface);
                            captured = true;
                        break;
                    }
                }
            }
            
                if !captured {
                    log::warn!("Failed to start capture on any interface");
                }
            } else {
                log::info!("Found {} Docker network interface(s) - monitoring all of them", network_interfaces.len());
                
                // Capture on all Docker interfaces
        for interface in network_interfaces {
            log::info!("Attempting to monitor interface: {}", interface);
            let monitor_clone = Arc::new(NetworkMonitorService {
                docker_service: self.docker_service.clone(),
                db: self.db.clone(),
                captured_requests: Arc::clone(&self.captured_requests),
                pending_requests: Arc::clone(&self.pending_requests),
            });
            let interface_clone = interface.clone();
            tokio::spawn(async move {
                if let Err(e) = monitor_clone.capture_packets(&interface_clone).await {
                    log::warn!("Failed to capture packets on {}: {}", interface_clone, e);
                }
            });
        }
            }
        }
        
        log::warn!("Note: Network packet capture requires elevated privileges:");
        #[cfg(target_os = "linux")]
        log::warn!("  Linux: root or CAP_NET_RAW capability (use --cap-add=NET_RAW in Docker)");
        #[cfg(target_os = "macos")]
        log::warn!("  macOS: root or admin privileges");
        #[cfg(target_os = "windows")]
        log::warn!("  Windows: Administrator privileges and Npcap/WinPcap installed");

        Ok(())
    }
    
    /// Detect if the application is running inside a Docker container
    async fn detect_if_in_docker(&self) -> bool {
        // Check for /.dockerenv file (Docker creates this in containers)
        if std::path::Path::new("/.dockerenv").exists() {
            return true;
        }
        
        // Check /proc/self/cgroup for Docker (Linux only)
        #[cfg(target_os = "linux")]
        {
            if let Ok(cgroup_content) = std::fs::read_to_string("/proc/self/cgroup") {
                if cgroup_content.contains("docker") || cgroup_content.contains("containerd") {
                    return true;
                }
            }
        }
        
        false
    }

    /// Get Docker network interfaces (bridge networks)
    /// Works on Linux, macOS, and Windows using pcap's Device::list()
    async fn get_docker_network_interfaces(&self) -> Result<Vec<String>> {
        #[cfg(feature = "network-capture")]
        {
            use pcap::Device;
            
            // Use pcap to list all available network interfaces
            // This works cross-platform (Linux, macOS, Windows)
            let devices = match Device::list() {
                Ok(devices) => devices,
                Err(e) => {
                    log::warn!("Failed to list network devices: {}. Packet capture may not be available.", e);
                    return Ok(vec![]);
                }
            };
            
            let possible_prefixes = vec![
                "docker",           // docker0, docker1, etc.
                "br-",              // Bridge networks (br-xxx)
                "veth",             // Virtual ethernet pairs
                "vEthernet",        // Windows Hyper-V virtual ethernet
                "DockerNAT",        // Windows Docker Desktop
                "vmnet",            // macOS/VMware virtual network
                "bridge",           // Generic bridge interfaces
            ];
            
            let mut interfaces = Vec::new();
            
            for device in devices {
                let name = device.name;
                
                // Check if interface name matches Docker-related patterns
                let is_docker_interface = possible_prefixes.iter().any(|prefix| {
                    name.to_lowercase().starts_with(&prefix.to_lowercase())
                });
                
                if is_docker_interface {
                    log::debug!("Found potential Docker interface: {}", name);
                    interfaces.push(name);
                }
            }
            
            // If no Docker-specific interfaces found, list all interfaces for manual selection
            // On macOS/Windows, we might need to capture on the main network interface
            if interfaces.is_empty() {
                log::info!("No Docker-specific interfaces found. Listing all available interfaces:");
                if let Ok(devices) = Device::list() {
                    for device in devices {
                        log::info!("  - {} ({})", device.name, 
                            device.desc.as_ref().unwrap_or(&"No description".to_string()));
                    }
                }
                
                // On macOS/Windows, try to use the default interface or loopback
                // User can configure which interface to monitor
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                {
                    log::info!("On macOS/Windows, you may need to specify the interface manually.");
                    log::info!("Common interfaces to try: 'en0' (macOS), 'Ethernet' (Windows), or 'any'");
                }
            }
            
            Ok(interfaces)
        }
        
        #[cfg(not(feature = "network-capture"))]
        {
            log::info!("Network capture feature not enabled. Install Npcap/WinPcap (Windows) or libpcap (Linux/macOS) and rebuild with --features network-capture");
            Ok(vec![])
        }
    }

    /// Parse HTTP request from packet data
    fn parse_http_request(&self, packet_data: &[u8]) -> Option<ParsedHttpRequest> {
        use httparse::{Request, Status};
        
        let mut req_headers = [httparse::EMPTY_HEADER; 64];
        let mut req = Request::new(&mut req_headers);
        
        match req.parse(packet_data) {
            Ok(Status::Complete(_)) => {
                Some(ParsedHttpRequest {
                    method: req.method?.to_string(),
                    path: req.path?.to_string(),
                    status: None,
                    response_time_ms: None,
                })
            }
            Ok(Status::Partial) | Err(_) => None,
        }
    }

    /// Parse HTTP response from packet data
    fn parse_http_response(&self, packet_data: &[u8]) -> Option<ParsedHttpResponse> {
        use httparse::{Response, Status};
        
        let mut resp_headers = [httparse::EMPTY_HEADER; 64];
        let mut resp = Response::new(&mut resp_headers);
        
        match resp.parse(packet_data) {
            Ok(Status::Complete(_)) => {
                Some(ParsedHttpResponse {
                    status: resp.code,
                })
            }
            Ok(Status::Partial) | Err(_) => None,
        }
    }

    /// Handle HTTP request - store as pending and wait for response
    async fn handle_http_request(
        &self,
        connection_id: &str,
        container_id: &str,
        container_name: &str,
        method: String,
        path: String,
    ) {
        let pending = PendingRequest {
            container_id: container_id.to_string(),
            container_name: container_name.to_string(),
            method: method.clone(),
            endpoint: path.clone(),
            request_timestamp: Utc::now(),
        };

        let mut pending_map = self.pending_requests.write().await;
        
        // Clean up old pending requests (older than 30 seconds) before inserting
        let cutoff = Utc::now() - chrono::Duration::seconds(30);
        let before_cleanup = pending_map.len();
        pending_map.retain(|_, req| req.request_timestamp > cutoff);
        let after_cleanup = pending_map.len();
        if before_cleanup != after_cleanup {
            log::debug!("Cleaned up {} expired pending requests (kept {})", 
                before_cleanup - after_cleanup, after_cleanup);
        }
        
        pending_map.insert(connection_id.to_string(), pending);
        
        if container_id == "unknown" {
            log::info!("📝 Stored pending HTTP request: {} {} (connection: {}, container: unknown - will try to match on response)", 
                method, path, connection_id);
        } else {
            log::info!("📝 Stored pending HTTP request: {} {} (connection: {}, container: {} ({})", 
                method, path, connection_id, &container_id[..12], container_name);
        }
    }

    /// Handle HTTP response - match with pending request and create complete HttpRequest
    async fn handle_http_response(
        &self,
        connection_id: &str,
        container_id: &str,
        container_name: &str,
        status_code: Option<u16>,
    ) {
        let mut pending_map = self.pending_requests.write().await;
        
        if let Some(mut pending) = pending_map.remove(connection_id) {
            // If container was unknown in the request, use the one from the response
            if pending.container_id == "unknown" && container_id != "unknown" {
                log::info!("Updating container info for connection {}: {} -> {}", 
                    connection_id, pending.container_id, container_id);
                pending.container_id = container_id.to_string();
                pending.container_name = container_name.to_string();
            }
            
            // Use container from pending request (which may have been updated above)
            let final_container_id = if pending.container_id != "unknown" {
                pending.container_id.clone()
            } else {
                container_id.to_string()
            };
            
            let final_container_name = if pending.container_name != "unknown" {
                pending.container_name.clone()
            } else {
                container_name.to_string()
            };
            
            let response_timestamp = Utc::now();
            let latency_ms = (response_timestamp - pending.request_timestamp)
                .num_milliseconds() as f64;

            let request = HttpRequest {
                container_id: final_container_id.clone(),
                container_name: final_container_name.clone(),
                endpoint: pending.endpoint.clone(),
                method: pending.method.clone(),
                http_status: status_code.unwrap_or(200),
                response_time_ms: latency_ms.max(0.0),
                timestamp: pending.request_timestamp,
            };
            
            // Only store if we have a valid container ID
            if final_container_id != "unknown" {
                log::info!("✅ Captured HTTP request: {} {} {} {}ms from container {} ({})", 
                    request.method, request.endpoint, request.http_status, 
                    request.response_time_ms, &final_container_id[..12], final_container_name);
                
                // Try to insert directly into database if available
                if let Some(db) = &self.db {
                    if let Err(e) = self.insert_http_request_to_db(db, &request).await {
                        log::warn!("Failed to insert HTTP request directly to database: {}. Storing in memory as fallback.", e);
                        // Fallback to in-memory storage
                        self.store_request(final_container_id, request).await;
                    } else {
                        log::info!("💾 Successfully inserted HTTP request directly into database: {} {} {} ({}ms)", 
                            request.method, request.endpoint, request.http_status, request.response_time_ms);
                    }
                } else {
                    // No database connection - store in memory for later collection
                    self.store_request(final_container_id, request).await;
                }
            } else {
                log::warn!("⚠️ HTTP request/response matched but container still unknown - skipping storage (connection: {})", connection_id);
            }
        } else {
            log::debug!("Received HTTP response for unknown connection: {} (no pending request found)", connection_id);
            // Log pending requests for debugging
            let pending_count = pending_map.len();
            if pending_count > 0 {
                log::debug!("Currently have {} pending requests (connection IDs: {:?})", 
                    pending_count, 
                    pending_map.keys().take(5).collect::<Vec<_>>());
            }
        }
    }

    /// Extract connection info and HTTP payload from packet
    /// Returns: (connection_id, http_payload, is_response)
    /// 
    /// Note: On Linux "any" interface, packets use SLL (Socket Layer Link) header (16 bytes)
    /// instead of Ethernet header (14 bytes). We need to detect this.
    #[cfg(feature = "network-capture")]
    async fn extract_connection_info<'a>(&self, packet: &'a pcap::Packet<'_>) -> Option<(String, &'a [u8], bool)> {
        if packet.data.len() < 34 {
            return None;
        }

        // Detect packet type: SLL (Linux "any" interface) or Ethernet
        // SLL header starts with 0x00 0x00 (packet type field)
        // Ethernet header starts with MAC addresses (not 0x00 0x00 typically)
        let is_sll = packet.data[0] == 0x00 && packet.data[1] == 0x00;
        
        let (ip_header_start, ip_protocol_offset, src_ip_offset, dst_ip_offset) = if is_sll {
            // SLL header: 16 bytes
            // IP header starts at byte 16
            // IP protocol is at byte 9 of IP header = byte 25 of packet
            // Source IP: bytes 12-15 of IP header = bytes 28-31 of packet
            // Dest IP: bytes 16-19 of IP header = bytes 32-35 of packet
            (16, 25, 28, 32)
        } else {
            // Ethernet header: 14 bytes
            // IP header starts at byte 14
            // IP protocol is at byte 9 of IP header = byte 23 of packet
            // Source IP: bytes 12-15 of IP header = bytes 26-29 of packet
            // Dest IP: bytes 16-19 of IP header = bytes 30-33 of packet
            (14, 23, 26, 30)
        };

        if packet.data.len() < dst_ip_offset + 4 {
            return None;
        }

        // Check IP protocol
        let ip_protocol = packet.data[ip_protocol_offset];
        if ip_protocol != 6 {
            // Not TCP (protocol 6)
            return None;
        }

        // Extract IP addresses
        let src_ip = format!("{}.{}.{}.{}", 
            packet.data[src_ip_offset], packet.data[src_ip_offset + 1], 
            packet.data[src_ip_offset + 2], packet.data[src_ip_offset + 3]);
        let dst_ip = format!("{}.{}.{}.{}", 
            packet.data[dst_ip_offset], packet.data[dst_ip_offset + 1], 
            packet.data[dst_ip_offset + 2], packet.data[dst_ip_offset + 3]);

        // Extract TCP ports
        // TCP header starts after IP header (IP header is typically 20 bytes, but can have options)
        // For simplicity, assume IP header is 20 bytes (no options)
        let tcp_start = ip_header_start + 20;
        if packet.data.len() < tcp_start + 4 {
            return None;
        }
        let src_port = ((packet.data[tcp_start] as u16) << 8) | (packet.data[tcp_start + 1] as u16);
        let dst_port = ((packet.data[tcp_start + 2] as u16) << 8) | (packet.data[tcp_start + 3] as u16);

        // Determine if this is a response (packet going TO a server port)
        // This is a heuristic - we check if destination port is a common HTTP server port
        // In practice, we should track which IP is the container to determine direction more accurately
        let is_response = dst_port == 80 || dst_port == 443 || dst_port == 8080 || 
                         dst_port == 8443 || dst_port == 8000 || dst_port == 3000 || 
                         dst_port == 5000 || dst_port == 9000;

        // Create connection ID (bidirectional - same connection regardless of direction)
        let connection_id = if src_port < dst_port {
            format!("{}:{}:{}:{}", src_ip, src_port, dst_ip, dst_port)
        } else {
            format!("{}:{}:{}:{}", dst_ip, dst_port, src_ip, src_port)
        };

        // Extract HTTP payload (skip link layer + IP + TCP headers)
        // TCP header length is in the 4-bit header length field (byte 12 of TCP header, bits 4-7)
        let tcp_header_len_offset = tcp_start + 12;
        if packet.data.len() < tcp_header_len_offset + 1 {
            return None;
        }
        let tcp_header_len = ((packet.data[tcp_header_len_offset] & 0xF0) >> 4) * 4;
        let http_start = tcp_start + tcp_header_len as usize;
        
        if packet.data.len() > http_start {
            Some((connection_id, &packet.data[http_start..], is_response))
        } else {
            None
        }
    }

    /// Extract connection info (fallback when network-capture feature is disabled)
    #[cfg(not(feature = "network-capture"))]
    async fn extract_connection_info(&self, _packet: &[u8]) -> Option<(String, &[u8], bool)> {
        None
    }

    /// Insert HTTP request directly into database
    async fn insert_http_request_to_db(
        &self,
        db: &DatabaseConnection,
        request: &HttpRequest,
    ) -> Result<()> {
        use crate::entity::http_requests;
        
        let fixed_offset = FixedOffset::east_opt(0).unwrap();
        let timestamp = request.timestamp.with_timezone(&fixed_offset);
        
        let active_model = http_requests::ActiveModel {
            container_id: Set(request.container_id.clone()),
            container_name: Set(request.container_name.clone()),
            endpoint: Set(request.endpoint.clone()),
            method: Set(request.method.clone()),
            http_status: Set(request.http_status as i16),
            response_time_ms: Set(request.response_time_ms),
            timestamp: Set(timestamp),
            ..Default::default()
        };
        
        active_model
            .insert(db)
            .await
            .map_err(|e| anyhow::anyhow!("Database insert failed: {}", e))?;
        
        Ok(())
    }

    /// Store a captured HTTP request in memory (fallback when database is not available)
    pub async fn store_request(&self, container_id: String, request: HttpRequest) {
        let mut requests = self.captured_requests.write().await;
        let container_requests = requests.entry(container_id.clone()).or_insert_with(Vec::new);
        container_requests.push(request.clone());
        
        log::info!("📦 Stored HTTP request in memory: {} {} (container: {}, total stored: {})", 
            request.method, request.endpoint, &container_id[..12], container_requests.len());
        
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
    /// Cross-platform: Works on Linux, macOS, and Windows
    /// Uses existing pcap library - no need to implement packet capture from scratch
    async fn capture_packets(&self, interface: &str) -> Result<()> {
        #[cfg(feature = "network-capture")]
        {
            use pcap::{Capture, Device};
            
            // Find the device - works on all platforms
            let devices = Device::list()?;
            let device = devices
                .iter()
                .find(|d| d.name == interface || d.name.to_lowercase() == interface.to_lowercase())
                .ok_or_else(|| {
                    log::warn!("Interface '{}' not found. Available interfaces:", interface);
                    for d in &devices {
                        log::warn!("  - {} ({})", d.name, d.desc.as_ref().unwrap_or(&"No description".to_string()));
                    }
                    anyhow::anyhow!("Interface {} not found", interface)
                })?;
            
            log::info!("Attempting to capture on interface: {} ({})", 
                device.name, 
                device.desc.as_ref().unwrap_or(&"No description".to_string()));
            
            // Open capture - requires elevated privileges on all platforms
            // Linux: root or CAP_NET_RAW
            // macOS: root or admin privileges
            // Windows: Administrator or Npcap/WinPcap installed with proper permissions
            let mut cap = match Capture::from_device(device.name.as_str()) {
                Ok(cap) => {
                    // Configure capture settings
                    let capture = cap.promisc(true).snaplen(65535);
                    
                    // Open the capture
                    match capture.open() {
                        Ok(c) => c,
                        Err(e) => {
                            log::warn!("Failed to open capture on {}: {}", interface, e);
                            #[cfg(target_os = "linux")]
                            log::warn!("On Linux: Need root or CAP_NET_RAW capability");
                            #[cfg(target_os = "macos")]
                            log::warn!("On macOS: Need root or admin privileges");
                            #[cfg(target_os = "windows")]
                            log::warn!("On Windows: Need Administrator privileges and Npcap/WinPcap installed");
                            return Err(anyhow::anyhow!("Permission denied: {}. See logs for platform-specific requirements.", e));
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to create capture device: {}", e);
                    return Err(anyhow::anyhow!("Failed to create capture: {}", e));
                }
            };
            
            // Filter for HTTP traffic (ports 80, 8080, 8000, 3000, etc.)
            // BPF filter syntax works on all platforms
            // Also include common application ports
            let filter_str = "tcp port 80 or tcp port 443 or tcp port 8080 or tcp port 8443 or tcp port 8000 or tcp port 3000 or tcp port 5000 or tcp port 9000";
            if let Err(e) = cap.filter(filter_str, true) {
                log::warn!("Failed to set packet filter: {}. Capturing all TCP traffic.", e);
            } else {
                log::info!("Packet filter applied: {} (capturing HTTP/HTTPS traffic on common ports)", filter_str);
            }
            
            log::info!("Started capturing packets on {} (cross-platform)", interface);
            
            // Capture packets in a loop (blocking, so this runs in a separate task)
            // This works the same on Linux, macOS, and Windows
            let mut packet_count = 0u64;
            let mut tcp_packet_count = 0u64;
            let mut http_packet_count = 0u64;
            let mut last_log_time = std::time::Instant::now();
            
            log::info!("Starting packet capture loop on interface: {}", interface);
            
            loop {
                let packet = match cap.next_packet() {
                    Ok(p) => {
                        packet_count += 1;
                        // Log packet capture stats every 10 seconds
                        if last_log_time.elapsed().as_secs() >= 10 {
                            log::info!("Packet capture stats (last 10s): Total={}, TCP={}, HTTP={} on {}", 
                                packet_count, tcp_packet_count, http_packet_count, interface);
                            packet_count = 0;
                            tcp_packet_count = 0;
                            http_packet_count = 0;
                            last_log_time = std::time::Instant::now();
                        }
                        p
                    },
                    Err(pcap::Error::TimeoutExpired) => {
                        // Timeout is normal, continue
                        // Log if we haven't seen any packets for a while
                        if last_log_time.elapsed().as_secs() >= 30 && packet_count == 0 {
                            log::warn!("No packets captured in last 30 seconds on {}. This may indicate:", interface);
                            log::warn!("  1. No network traffic on this interface");
                            log::warn!("  2. Worker container cannot see traffic from host/other containers");
                            log::warn!("  3. Consider using host network mode or ensuring worker is on same network");
                            last_log_time = std::time::Instant::now();
                        }
                        continue;
                    }
                    Err(e) => {
                        log::error!("Error capturing packet: {}", e);
                        break;
                    }
                };
                
                // Check if it's TCP (increment counter)
                // Detect SLL vs Ethernet header
                if packet.data.len() >= 34 {
                    let is_sll = packet.data[0] == 0x00 && packet.data[1] == 0x00;
                    let ip_protocol_offset = if is_sll { 25 } else { 23 };
                    if packet.data.len() > ip_protocol_offset {
                        let ip_protocol = packet.data[ip_protocol_offset];
                        if ip_protocol == 6 {
                            tcp_packet_count += 1;
                        }
                    }
                }
                
                // Extract connection info and HTTP data
                if let Some((connection_id, http_data, is_response)) = self.extract_connection_info(&packet).await {
                    http_packet_count += 1;
                    
                    // Try to parse as both request and response to determine actual direction
                    let parsed_request = self.parse_http_request(http_data);
                    let parsed_response = self.parse_http_response(http_data);
                    
                    // Use actual HTTP parsing results to determine direction (more reliable than port heuristics)
                    let is_http_request = parsed_request.is_some();
                    let is_http_response = parsed_response.is_some();
                    
                    log::info!("Extracted HTTP data from packet: connection_id={}, port_heuristic_is_response={}, data_len={}, is_http_request={}, is_http_response={}", 
                        connection_id, is_response, http_data.len(), is_http_request, is_http_response);
                    
                    // Try to find container for this packet
                    let container_match = self.find_container_for_packet(&packet).await;
                    
                    // Use actual HTTP parsing to determine direction
                    if is_http_response {
                        // This is an HTTP response - try to match with pending request
                        if let Some(parsed) = parsed_response {
                            if let Some(ref container_id) = container_match {
                                let container_name = self.docker_service
                                    .list_containers()
                                    .await
                                    .ok()
                                    .and_then(|containers| {
                                        containers
                                            .iter()
                                            .find(|c| c.id == *container_id)
                                            .map(|c| c.name.clone())
                                    })
                                    .unwrap_or_else(|| "unknown".to_string());
                                
                                log::debug!("Processing HTTP response for container {} (connection: {})", container_id, connection_id);
                                log::info!("Parsed HTTP response: status={:?} for container {}", parsed.status, container_id);
                                self.handle_http_response(
                                    &connection_id,
                                    container_id,
                                    &container_name,
                                    parsed.status,
                                ).await;
                            } else {
                                // Response arrived but no container match - try to match with pending request anyway
                                // The pending request might have container info
                                log::debug!("HTTP response received but container not matched - trying to match with pending request (connection: {})", connection_id);
                                self.handle_http_response(
                                    &connection_id,
                                    "unknown",
                                    "unknown",
                                    parsed.status,
                                ).await;
                            }
                        } else {
                            log::debug!("Failed to parse HTTP response data (len={})", http_data.len());
                        }
                    } else if is_http_request {
                        // This is an HTTP request - store as pending and wait for response
                        if let Some(parsed) = parsed_request {
                            if let Some(ref container_id) = container_match {
                                let container_name = self.docker_service
                                    .list_containers()
                                    .await
                                    .ok()
                                    .and_then(|containers| {
                                        containers
                                            .iter()
                                            .find(|c| c.id == *container_id)
                                            .map(|c| c.name.clone())
                                    })
                                    .unwrap_or_else(|| "unknown".to_string());
                                
                                log::debug!("Processing HTTP request for container {} (connection: {})", container_id, connection_id);
                                log::info!("Parsed HTTP request: {} {} for container {}", parsed.method, parsed.path, container_id);
                                self.handle_http_request(
                                    &connection_id,
                                    container_id,
                                    &container_name,
                                    parsed.method,
                                    parsed.path,
                                ).await;
                            } else {
                                // Store request even if container not matched - we'll try to match when response arrives
                                log::info!("Parsed HTTP request: {} {} but container not matched yet (connection: {}) - storing as pending", 
                                    parsed.method, parsed.path, connection_id);
                                self.handle_http_request(
                                    &connection_id,
                                    "unknown",
                                    "unknown",
                                    parsed.method,
                                    parsed.path,
                                ).await;
                            }
                        } else {
                            log::debug!("Failed to parse HTTP request data (len={})", http_data.len());
                        }
                    } else {
                        log::debug!("Packet contains HTTP data but neither request nor response could be parsed (len={})", http_data.len());
                    }
                    
                    // Log container matching failure for debugging
                    if container_match.is_none() && packet.data.len() >= 34 {
                        let is_sll = packet.data[0] == 0x00 && packet.data[1] == 0x00;
                        let (src_ip_offset, dst_ip_offset) = if is_sll { (28, 32) } else { (26, 30) };
                        if packet.data.len() >= dst_ip_offset + 4 {
                            let src_ip = format!("{}.{}.{}.{}", 
                                packet.data[src_ip_offset], packet.data[src_ip_offset + 1], 
                                packet.data[src_ip_offset + 2], packet.data[src_ip_offset + 3]);
                            let dst_ip = format!("{}.{}.{}.{}", 
                                packet.data[dst_ip_offset], packet.data[dst_ip_offset + 1], 
                                packet.data[dst_ip_offset + 2], packet.data[dst_ip_offset + 3]);
                            log::debug!("Could not match packet to container - Source IP: {}, Dest IP: {} (connection: {})", 
                                src_ip, dst_ip, connection_id);
                        }
                    }
                } else {
                    // Packet didn't contain HTTP data or wasn't TCP
                    log::trace!("Packet did not contain HTTP data or was not TCP");
                }
            }
            
            Ok(())
        }
        
        #[cfg(not(feature = "network-capture"))]
        {
            log::warn!("Network capture feature not enabled. Cannot capture packets on {}", interface);
            log::info!("Install Npcap/WinPcap (Windows) or libpcap (Linux/macOS) and rebuild with --features network-capture");
            Err(anyhow::anyhow!("Network capture feature not enabled"))
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

    /// Try to capture on a specific interface (helper method)
    async fn try_capture_on_interface(&self, interface: &str) -> Result<()> {
        let monitor_clone = Arc::new(NetworkMonitorService {
            docker_service: self.docker_service.clone(),
            db: self.db.clone(),
            captured_requests: Arc::clone(&self.captured_requests),
            pending_requests: Arc::clone(&self.pending_requests),
        });
        
        let interface_clone = interface.to_string();
        tokio::spawn(async move {
            if let Err(e) = monitor_clone.capture_packets(&interface_clone).await {
                log::debug!("Failed to capture on {}: {}", interface_clone, e);
            }
        });
        
        Ok(())
    }

    /// Find container ID for a packet by matching IP addresses
    /// Works for both scenarios:
    /// 1. Running locally: matches packets to/from container IPs
    /// 2. Running in Docker: matches packets on Docker network
    #[cfg(feature = "network-capture")]
    async fn find_container_for_packet(&self, packet: &pcap::Packet<'_>) -> Option<String> {
        // Extract IP addresses from packet
        // On Linux "any" interface, packets use SLL header (16 bytes) instead of Ethernet (14 bytes)
        if packet.data.len() < 34 {
            return None;
        }
        
        // Detect packet type: SLL (Linux "any" interface) or Ethernet
        let is_sll = packet.data[0] == 0x00 && packet.data[1] == 0x00;
        let (src_ip_offset, dst_ip_offset) = if is_sll {
            // SLL: Source IP at bytes 28-31, Dest IP at bytes 32-35
            (28, 32)
        } else {
            // Ethernet: Source IP at bytes 26-29, Dest IP at bytes 30-33
            (26, 30)
        };
        
        if packet.data.len() < dst_ip_offset + 4 {
            return None;
        }
        
        // Extract source and destination IP addresses from IP header
        let src_ip = format!("{}.{}.{}.{}", 
            packet.data[src_ip_offset], packet.data[src_ip_offset + 1], 
            packet.data[src_ip_offset + 2], packet.data[src_ip_offset + 3]);
        let dst_ip = format!("{}.{}.{}.{}", 
            packet.data[dst_ip_offset], packet.data[dst_ip_offset + 1], 
            packet.data[dst_ip_offset + 2], packet.data[dst_ip_offset + 3]);
        
        log::debug!("Packet IPs - Source: {}, Dest: {}", src_ip, dst_ip);
        
        // Skip localhost/loopback traffic (unless we're specifically looking for it)
        if src_ip == "127.0.0.1" || dst_ip == "127.0.0.1" {
            log::debug!("Skipping localhost traffic");
            return None;
        }
        
        // Cache container IP mappings to avoid repeated Docker API calls
        // This is a simple in-memory cache - in production you might want a more sophisticated cache
        use std::sync::OnceLock;
        static CONTAINER_IP_CACHE: OnceLock<tokio::sync::RwLock<std::collections::HashMap<String, Vec<String>>>> = OnceLock::new();
        
        let cache = CONTAINER_IP_CACHE.get_or_init(|| {
            tokio::sync::RwLock::new(std::collections::HashMap::new())
        });
        
        // Check cache first
        {
            let cache_read = cache.read().await;
            for (container_id, ips) in cache_read.iter() {
                if ips.contains(&src_ip) || ips.contains(&dst_ip) {
                    log::info!("Matched packet to container {} from cache (IP: {})", container_id, 
                        if ips.contains(&src_ip) { &src_ip } else { &dst_ip });
                    return Some(container_id.clone());
                }
            }
        }
        
        // Cache miss - query Docker API
        // Refresh cache periodically (every 30 seconds would be reasonable, but for now we'll refresh on miss)
        log::info!("Cache miss - querying Docker API for container IPs matching {} or {}", src_ip, dst_ip);
        
        // Add timeout to prevent hanging
        let containers_result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            self.docker_service.list_containers()
        ).await;
        
        let containers = match containers_result {
            Ok(Ok(containers)) => containers,
            Ok(Err(e)) => {
                log::warn!("Failed to list containers for IP matching: {}", e);
                return None;
            }
            Err(_) => {
                log::warn!("Timeout while querying Docker API for container IPs");
                return None;
            }
        };
        
        if !containers.is_empty() {
            let mut cache_write = cache.write().await;
            cache_write.clear(); // Simple refresh strategy - clear and rebuild
            
            let mut all_container_ips = Vec::new();
            
            let container_count = containers.len();
            log::info!("Checking {} containers for IP match", container_count);
            
            for container in &containers {
                // Get network info for this container
                match self.docker_service.get_container_network_info(&container.id).await {
                    Ok(network_info) => {
                        let container_ips = network_info.ip_addresses.clone();
                        log::info!("Container {} (name: {}) has IPs: {:?}", 
                            &container.id[..12], container.name, container_ips);
                        all_container_ips.extend(container_ips.iter().cloned());
                        
                        // Store in cache
                        cache_write.insert(container.id.clone(), container_ips.clone());
                        
                        // Check if packet IP matches any container IP
                        for container_ip in &container_ips {
                            if container_ip == &src_ip || container_ip == &dst_ip {
                                log::info!("✅ Matched packet to container {} (name: {}) with IP: {}", 
                                    &container.id[..12], container.name, container_ip);
                                return Some(container.id.clone());
                            }
                        }
                    }
                    Err(e) => {
                        log::debug!("Failed to get network info for container {}: {}", &container.id[..12], e);
                    }
                }
            }
            
            log::warn!("❌ No container IP matched. Packet IPs: {} / {}. Known container IPs ({} total): {:?}", 
                src_ip, dst_ip, all_container_ips.len(), all_container_ips);
            
            // Also log which containers we checked
            log::debug!("Checked {} containers for IP matching", container_count);
        } else {
            log::warn!("Failed to list containers for IP matching");
        }
        
        None
    }
    
    /// Find container ID for a packet by matching IP addresses (fallback when network-capture feature is disabled)
    #[cfg(not(feature = "network-capture"))]
    async fn find_container_for_packet(&self, _packet: &[u8]) -> Option<String> {
        None
    }
}

/// Parsed HTTP request from network packet
struct ParsedHttpRequest {
    method: String,
    path: String,
    status: Option<u16>,
    response_time_ms: Option<f64>,
}

/// Parsed HTTP response from network packet
struct ParsedHttpResponse {
    status: Option<u16>,
}

