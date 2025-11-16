import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  ContainerInfo,
  ContainerStats,
  ContainerLog,
  DataPoint,
  ServiceMap,
} from '../types';
import {
  fetchContainers,
  fetchContainerStats,
  fetchContainerLogs,
  fetchServiceMap,
} from '../services/api';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';
import { Input } from '../components/ui/input';
import { Separator } from '../components/ui/separator';
import MetricsChart from '../components/MetricsChart';
import { formatBytes, formatPercent, formatDate } from '../utils/formatting';
import { Search, X } from 'lucide-react';

const MAX_HISTORY = 60;

const APM = () => {
  const [containers, setContainers] = useState<ContainerInfo[]>([]);
  const [selectedServiceId, setSelectedServiceId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [serviceStats, setServiceStats] = useState<ContainerStats | null>(null);
  const [serviceLogs, setServiceLogs] = useState<ContainerLog[]>([]);
  const [serviceMap, setServiceMap] = useState<ServiceMap | null>(null);
  const [historicalData, setHistoricalData] = useState<DataPoint[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [logLimit, setLogLimit] = useState(100);

  const intervalRef = useRef<number | null>(null);

  // Load containers list
  useEffect(() => {
    const loadContainers = async () => {
      try {
        const data = await fetchContainers();
        setContainers(data);
      } catch (err) {
        console.error('Failed to load containers:', err);
      }
    };
    loadContainers();
  }, []);

  // Filter containers by search query
  const filteredContainers = containers.filter((container) => {
    if (!searchQuery.trim()) return true;
    const query = searchQuery.toLowerCase();
    return (
      container.name.toLowerCase().includes(query) ||
      container.image.toLowerCase().includes(query) ||
      container.id.toLowerCase().includes(query)
    );
  });

  // Load service details when selected
  const loadServiceDetails = useCallback(async (serviceId: string) => {
    try {
      setLoading(true);
      setError(null);

      // Load stats
      const stats = await fetchContainerStats(serviceId);
      setServiceStats(stats);

      // Load logs
      const logs = await fetchContainerLogs(serviceId, logLimit);
      setServiceLogs(logs);

      // Load service map for this service (to show connections)
      const map = await fetchServiceMap(serviceId);
      setServiceMap(map);

      // Update historical data
      const timestamp = new Date(stats.timestamp).getTime() / 1000;
      setHistoricalData((prev) => {
        const newData = [...prev];
        const networkKb = (stats.network_rx_bytes + stats.network_tx_bytes) / 1024;
        const dataPoint: DataPoint = {
          timestamp,
          cpu: stats.cpu_usage_percent,
          memory: stats.memory_usage_percent,
          network: networkKb,
        };
        newData.push(dataPoint);
        if (newData.length > MAX_HISTORY) {
          newData.shift();
        }
        return newData;
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load service details');
      console.error('Error loading service details:', err);
    } finally {
      setLoading(false);
    }
  }, [logLimit]);

  // Auto-refresh selected service stats
  useEffect(() => {
    if (selectedServiceId) {
      loadServiceDetails(selectedServiceId);
      
      intervalRef.current = window.setInterval(() => {
        loadServiceDetails(selectedServiceId);
      }, 2000);

      return () => {
        if (intervalRef.current) {
          clearInterval(intervalRef.current);
        }
      };
    }
  }, [selectedServiceId, loadServiceDetails]);

  const handleServiceSelect = (serviceId: string) => {
    setSelectedServiceId(serviceId);
    setHistoricalData([]); // Reset history when switching services
  };

  const selectedService = containers.find((c) => c.id === selectedServiceId);

  const getStatusVariant = (status: string): 'success' | 'destructive' | 'warning' | 'secondary' => {
    const statusLower = status.toLowerCase();
    if (statusLower.includes('running') || statusLower.includes('up')) return 'success';
    if (statusLower.includes('exited') || statusLower.includes('stopped')) return 'destructive';
    if (statusLower.includes('created') || statusLower.includes('paused')) return 'warning';
    return 'secondary';
  };

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-gray-900 mb-1">Application Performance Monitoring</h1>
        <p className="text-sm text-gray-600">Monitor and analyze individual service performance</p>
      </div>

      {/* Service Search and Selection */}
      <Card className="mb-6">
        <CardHeader>
          <CardTitle className="text-lg">Select Service</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="relative">
            <Search className="absolute left-3 top-2.5 h-5 w-5 text-gray-400" />
            <Input
              type="text"
              placeholder="Search services by name, image, or ID..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-10 pr-10"
            />
            {searchQuery && (
              <button
                onClick={() => setSearchQuery('')}
                className="absolute right-3 top-2.5 text-gray-400 hover:text-gray-600"
              >
                <X className="h-5 w-5" />
              </button>
            )}
          </div>

          {filteredContainers.length > 0 && (
            <div className="mt-4 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3 max-h-64 overflow-y-auto">
              {filteredContainers.map((container) => (
                <button
                  key={container.id}
                  onClick={() => handleServiceSelect(container.id)}
                  className={`p-3 text-left rounded-lg border-2 transition-all ${
                    selectedServiceId === container.id
                      ? 'border-blue-500 bg-blue-50'
                      : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                  }`}
                >
                  <div className="font-semibold text-gray-900">{container.name}</div>
                  <div className="text-xs text-gray-500 truncate mt-1">{container.image}</div>
                  <div className="mt-2">
                    <Badge variant={getStatusVariant(container.status)} className="text-xs">
                      {container.status}
                    </Badge>
                  </div>
                </button>
              ))}
            </div>
          )}

          {filteredContainers.length === 0 && searchQuery && (
            <div className="mt-4 text-center text-gray-500 text-sm">
              No services found matching &quot;{searchQuery}&quot;
            </div>
          )}
        </CardContent>
      </Card>

      {/* Service Details */}
      {selectedServiceId && selectedService && (
        <>
          {error && (
            <div className="mb-4 bg-red-50 text-red-800 p-4 rounded border-l-4 border-red-800">
              <strong>Error:</strong> {error}
            </div>
          )}

          {loading && (
            <div className="mb-4 p-4 text-center text-gray-600 bg-white rounded-lg">
              Loading service details...
            </div>
          )}

          {/* Service Overview */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6 mb-6">
            {/* Metrics Chart */}
            <div className="lg:col-span-2">
              <Card>
                <CardHeader>
                  <CardTitle className="text-lg">Performance Metrics</CardTitle>
                </CardHeader>
                <CardContent>
                  {historicalData.length > 0 ? (
                    <MetricsChart data={historicalData} width={600} height={200} />
                  ) : (
                    <div className="h-[200px] flex items-center justify-center text-gray-500">
                      Collecting metrics...
                    </div>
                  )}
                </CardContent>
              </Card>
            </div>

            {/* Current Stats */}
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Current Status</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                {serviceStats ? (
                  <>
                    <div>
                      <div className="text-xs text-gray-500 mb-1">CPU Usage</div>
                      <div className="text-2xl font-bold">{formatPercent(serviceStats.cpu_usage_percent)}</div>
                    </div>
                    <Separator />
                    <div>
                      <div className="text-xs text-gray-500 mb-1">Memory Usage</div>
                      <div className="text-2xl font-bold">{formatBytes(serviceStats.memory_usage_bytes)}</div>
                      <div className="text-sm text-gray-600">
                        of {formatBytes(serviceStats.memory_limit_bytes)} ({formatPercent(serviceStats.memory_usage_percent)})
                      </div>
                    </div>
                    <Separator />
                    <div>
                      <div className="text-xs text-gray-500 mb-1">Network RX</div>
                      <div className="text-lg font-semibold">{formatBytes(serviceStats.network_rx_bytes)}</div>
                    </div>
                    <div>
                      <div className="text-xs text-gray-500 mb-1">Network TX</div>
                      <div className="text-lg font-semibold">{formatBytes(serviceStats.network_tx_bytes)}</div>
                    </div>
                  </>
                ) : (
                  <div className="text-gray-500 text-sm">No stats available</div>
                )}
              </CardContent>
            </Card>
          </div>

          {/* Request Monitoring */}
          <Card className="mb-6">
            <CardHeader>
              <CardTitle className="text-lg">Request Monitoring</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="text-sm text-gray-600 mb-4">
                  Network activity and request patterns for this service
                </div>
                
                {serviceStats ? (
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div className="p-3 bg-gray-50 rounded-lg">
                      <div className="text-xs text-gray-500 mb-1">Total Requests (RX)</div>
                      <div className="text-lg font-semibold">
                        {formatBytes(serviceStats.network_rx_bytes)}
                      </div>
                      <div className="text-xs text-gray-500 mt-1">Data received</div>
                    </div>
                    <div className="p-3 bg-gray-50 rounded-lg">
                      <div className="text-xs text-gray-500 mb-1">Total Responses (TX)</div>
                      <div className="text-lg font-semibold">
                        {formatBytes(serviceStats.network_tx_bytes)}
                      </div>
                      <div className="text-xs text-gray-500 mt-1">Data sent</div>
                    </div>
                    <div className="p-3 bg-gray-50 rounded-lg">
                      <div className="text-xs text-gray-500 mb-1">Total Network</div>
                      <div className="text-lg font-semibold">
                        {formatBytes(serviceStats.network_rx_bytes + serviceStats.network_tx_bytes)}
                      </div>
                      <div className="text-xs text-gray-500 mt-1">Total traffic</div>
                    </div>
                  </div>
                ) : (
                  <div className="text-gray-500 text-sm">No network stats available</div>
                )}

                <Separator className="my-4" />

                <div className="text-sm text-gray-500">
                  <p className="mb-2">
                    <strong>Note:</strong> Detailed endpoint monitoring (HTTP methods, status codes, endpoints) 
                    requires request interception middleware or proxy integration.
                  </p>
                  <p>
                    Current monitoring shows aggregate network traffic. To track individual requests, 
                    consider integrating with:
                  </p>
                  <ul className="list-disc list-inside mt-2 space-y-1 text-xs">
                    <li>API Gateway or reverse proxy (nginx, traefik)</li>
                    <li>Application-level request logging</li>
                    <li>Service mesh (Istio, Linkerd)</li>
                  </ul>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Service Connections */}
          {serviceMap && serviceMap.edges.length > 0 && (
            <Card className="mb-6">
              <CardHeader>
                <CardTitle className="text-lg">Service Connections</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {serviceMap.edges.map((edge, idx) => {
                    const targetNode = serviceMap.nodes.find((n) => n.container_id === edge.to);
                    const sourceNode = serviceMap.nodes.find((n) => n.container_id === edge.from);
                    const isOutgoing = edge.from === selectedServiceId;
                    const connectedService = isOutgoing ? targetNode : sourceNode;

                    if (!connectedService) return null;

                    return (
                      <div
                        key={idx}
                        className="p-3 border border-gray-200 rounded-lg flex items-center justify-between hover:bg-gray-50 transition-colors"
                      >
                        <div className="flex items-center gap-3">
                          <div className="text-sm">
                            <span className="font-medium">{isOutgoing ? '→' : '←'}</span>{' '}
                            <span className="font-semibold">{connectedService.container_name}</span>
                          </div>
                          <Badge variant="secondary" className="text-xs">
                            {edge.connection_type.replace('_', ' ')}
                          </Badge>
                          <span className="text-xs text-gray-500">
                            {(edge.confidence * 100).toFixed(0)}% confidence
                          </span>
                        </div>
                        {edge.evidence.length > 0 && (
                          <div className="text-xs text-gray-400 max-w-xs truncate">
                            {edge.evidence[0]}
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </CardContent>
            </Card>
          )}

          {/* Service Logs */}
          <Card>
            <CardHeader>
              <div className="flex items-center justify-between">
                <CardTitle className="text-lg">Service Logs</CardTitle>
                <div className="flex items-center gap-2">
                  <Input
                    type="number"
                    placeholder="Limit"
                    min="1"
                    max="10000"
                    value={logLimit}
                    onChange={(e) => setLogLimit(parseInt(e.target.value, 10) || 100)}
                    className="w-24"
                  />
                  <Button
                    size="sm"
                    onClick={() => selectedServiceId && loadServiceDetails(selectedServiceId)}
                  >
                    Refresh
                  </Button>
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="max-h-[400px] overflow-y-auto font-mono text-sm">
                {serviceLogs.length > 0 ? (
                  serviceLogs.map((log, index) => (
                    <div
                      key={`${log.container_id}-${log.timestamp}-${index}`}
                      className={`py-1 border-b border-gray-100 flex gap-2.5 last:border-b-0 ${
                        log.stream === 'stderr' ? 'text-red-600' : 'text-gray-900'
                      }`}
                    >
                      <span className="text-gray-600 whitespace-nowrap flex-shrink-0">
                        [{formatDate(log.timestamp)}]
                      </span>
                      <span className="break-words">{log.log_line}</span>
                    </div>
                  ))
                ) : (
                  <div className="p-10 text-center text-gray-500">No logs available</div>
                )}
              </div>
            </CardContent>
          </Card>
        </>
      )}

      {!selectedServiceId && (
        <Card>
          <CardContent className="p-10 text-center">
            <p className="text-gray-500">Select a service from above to view detailed monitoring information</p>
          </CardContent>
        </Card>
      )}
    </div>
  );
};

export default APM;

