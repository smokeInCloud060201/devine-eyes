import { useMemo } from 'react';
import type { ContainerInfo, ContainerStats, DataPoint } from '../types';
import MetricsChart from './MetricsChart';
import { formatBytes, formatPercent } from '../utils/formatting';
import { Card, CardContent, CardHeader, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { Separator } from './ui/separator';

interface ContainerCardProps {
  container: ContainerInfo;
  stats: ContainerStats | null;
  historicalData: DataPoint[];
  onClick: () => void;
}

const ContainerCard = ({ container, stats, historicalData, onClick }: ContainerCardProps) => {
  const formattedStats = useMemo(() => {
    if (!stats) return null;

    return {
      cpu: formatPercent(stats.cpu_usage_percent),
      memoryUsed: formatBytes(stats.memory_usage_bytes),
      memoryLimit: formatBytes(stats.memory_limit_bytes),
      memoryPercent: formatPercent(stats.memory_usage_percent),
      networkRx: formatBytes(stats.network_rx_bytes),
      networkTx: formatBytes(stats.network_tx_bytes),
    };
  }, [stats]);

  const getStatusVariant = (status: string): 'success' | 'destructive' | 'warning' | 'secondary' => {
    const statusLower = status.toLowerCase();
    if (statusLower === 'running') return 'success';
    if (statusLower === 'exited' || statusLower === 'stopped') return 'destructive';
    if (statusLower === 'created' || statusLower === 'paused') return 'warning';
    return 'secondary';
  };

  return (
    <Card
      className="cursor-pointer transition-all duration-200 hover:-translate-y-0.5 hover:shadow-md"
      onClick={onClick}
    >
      <CardHeader className="pb-3">
        <CardTitle className="text-xl">{container.name}</CardTitle>
      </CardHeader>
      <CardContent>
        <div className="text-sm text-gray-600 mb-2">
          <span className="font-medium text-gray-900">ID:</span> {container.id.substring(0, 12)}...
        </div>
        <div className="text-sm text-gray-600 mb-2">
          <span className="font-medium text-gray-900">Image:</span> {container.image}
        </div>
        <div className="text-sm text-gray-600 mb-4">
          <span className="font-medium text-gray-900">Status:</span>{' '}
          <Badge variant={getStatusVariant(container.status)}>{container.status}</Badge>
        </div>

        {stats && formattedStats && (
          <>
            <Separator className="my-4" />
          {historicalData.length > 0 && (
            <div className="mb-4">
              <MetricsChart data={historicalData} width={300} height={120} />
              <div className="flex gap-4 justify-center text-sm text-gray-600 mt-1">
                <span className="flex items-center gap-1.5">
                  <span className="w-3 h-3 rounded bg-yellow-500"></span> CPU
                </span>
                <span className="flex items-center gap-1.5">
                  <span className="w-3 h-3 rounded bg-blue-500"></span> Memory
                </span>
                <span className="flex items-center gap-1.5">
                  <span className="w-3 h-3 rounded bg-green-500"></span> Network
                </span>
              </div>
            </div>
          )}
          <div className="text-sm">
            <div className="mb-2 text-gray-900">
              <strong className="text-gray-600 mr-1">CPU:</strong> {formattedStats.cpu}
            </div>
            <div className="mb-2 text-gray-900">
              <strong className="text-gray-600 mr-1">Memory:</strong> {formattedStats.memoryUsed} /{' '}
              {formattedStats.memoryLimit} ({formattedStats.memoryPercent})
            </div>
            <div className="mb-2 text-gray-900">
              <strong className="text-gray-600 mr-1">Network RX:</strong> {formattedStats.networkRx}
            </div>
            <div className="mb-2 text-gray-900">
              <strong className="text-gray-600 mr-1">Network TX:</strong> {formattedStats.networkTx}
            </div>
          </div>
          </>
        )}

        {!stats && (
          <>
            <Separator className="my-4" />
            <div className="text-gray-500 text-sm text-center">Stats unavailable</div>
          </>
        )}
      </CardContent>
    </Card>
  );
};

export default ContainerCard;
