import { useState, useEffect, useRef } from 'react';
import type { TotalStats } from '../types';
import { fetchTotalStats, connectSSEStats } from '../services/api';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { formatBytes, formatPercent } from '../utils/formatting';
import { Cpu, HardDrive, Network, Activity } from 'lucide-react';

const Dashboard = () => {
  const [totalStats, setTotalStats] = useState<TotalStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [sseConnected, setSseConnected] = useState(false);
  const sseRef = useRef<EventSource | null>(null);

  // Set up SSE connection for real-time stats
  useEffect(() => {
    const eventSource = connectSSEStats(
      (stats) => {
        setTotalStats(stats);
        setSseConnected(true);
        setError(null);
      },
      (err) => {
        console.error('SSE error:', err);
        setSseConnected(false);
      }
    );

    sseRef.current = eventSource;

    // Also fetch initial data
    const loadStats = async () => {
      try {
        setLoading(true);
        const stats = await fetchTotalStats();
        setTotalStats(stats);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load Docker stats');
      } finally {
        setLoading(false);
      }
    };

    loadStats();

    return () => {
      eventSource.close();
    };
  }, []);

  if (loading && !totalStats) {
    return (
      <div className="p-6">
        <div className="flex items-center justify-center h-96">
          <div className="text-gray-500">Loading Docker system status...</div>
        </div>
      </div>
    );
  }

  if (error && !totalStats) {
    return (
      <div className="p-6">
        <div className="bg-red-50 text-red-800 p-4 rounded border-l-4 border-red-800">
          <strong>Error:</strong> {error}
        </div>
      </div>
    );
  }

  if (!totalStats) {
    return null;
  }

  return (
    <div className="p-6">
      <div className="mb-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-semibold text-gray-900 mb-1">Docker System Status</h1>
            <p className="text-sm text-gray-600">Overall Docker daemon resource usage</p>
          </div>
          <div className="flex items-center space-x-4">
            <div className={`text-xs ${sseConnected ? 'text-green-600' : 'text-red-600'}`}>
              {sseConnected ? (
                <>🟢 Connected (Real-time)</>
              ) : (
                <>🔴 Disconnected</>
              )}
            </div>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {/* CPU Usage */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base font-medium">CPU Usage</CardTitle>
              <Cpu className="h-5 w-5 text-blue-600" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900 mb-2">
              {formatPercent(totalStats.total_cpu_usage_percent)}
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className="bg-blue-600 h-2 rounded-full transition-all"
                style={{ width: `${Math.min(totalStats.total_cpu_usage_percent, 100)}%` }}
              />
            </div>
          </CardContent>
        </Card>

        {/* Memory Usage */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base font-medium">Memory (RAM)</CardTitle>
              <Activity className="h-5 w-5 text-green-600" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900 mb-1">
              {formatBytes(totalStats.total_memory_usage_bytes)}
            </div>
            <div className="text-sm text-gray-600 mb-2">
              of {formatBytes(totalStats.total_memory_limit_bytes)}
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div
                className="bg-green-600 h-2 rounded-full transition-all"
                style={{ width: `${Math.min(totalStats.total_memory_usage_percent, 100)}%` }}
              />
            </div>
            <div className="text-xs text-gray-500 mt-1">
              {formatPercent(totalStats.total_memory_usage_percent)} used
            </div>
          </CardContent>
        </Card>

        {/* Network RX */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base font-medium">Network RX</CardTitle>
              <Network className="h-5 w-5 text-purple-600" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900">
              {formatBytes(totalStats.total_network_rx_bytes)}
            </div>
            <div className="text-sm text-gray-600 mt-1">Total received</div>
          </CardContent>
        </Card>

        {/* Network TX */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base font-medium">Network TX</CardTitle>
              <Network className="h-5 w-5 text-orange-600" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900">
              {formatBytes(totalStats.total_network_tx_bytes)}
            </div>
            <div className="text-sm text-gray-600 mt-1">Total sent</div>
          </CardContent>
        </Card>
      </div>

      {/* Additional System Info */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
        {/* Disk I/O Read */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base font-medium">Disk Read (ROM)</CardTitle>
              <HardDrive className="h-5 w-5 text-indigo-600" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900">
              {formatBytes(totalStats.total_block_read_bytes)}
            </div>
            <div className="text-sm text-gray-600 mt-1">Total bytes read</div>
          </CardContent>
        </Card>

        {/* Disk I/O Write */}
        <Card>
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="text-base font-medium">Disk Write (ROM)</CardTitle>
              <HardDrive className="h-5 w-5 text-pink-600" />
            </div>
          </CardHeader>
          <CardContent>
            <div className="text-3xl font-bold text-gray-900">
              {formatBytes(totalStats.total_block_write_bytes)}
            </div>
            <div className="text-sm text-gray-600 mt-1">Total bytes written</div>
          </CardContent>
        </Card>
      </div>

      {/* Container Count */}
      <div className="mt-6">
        <Card>
          <CardHeader>
            <CardTitle className="text-base font-medium">Container Summary</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-gray-900">
              {totalStats.total_containers} {totalStats.total_containers === 1 ? 'Container' : 'Containers'}
            </div>
            <div className="text-sm text-gray-600 mt-1">
              Total containers managed by Docker
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default Dashboard;

