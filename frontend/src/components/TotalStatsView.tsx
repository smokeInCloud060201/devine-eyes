import { useMemo } from 'react';
import type { TotalStats } from '../types';
import StatCard from './StatCard';
import { formatBytes, formatPercent } from '../utils/formatting';

interface TotalStatsViewProps {
  stats: TotalStats | null;
}

const TotalStatsView = ({ stats }: TotalStatsViewProps) => {
  const formattedStats = useMemo(() => {
    if (!stats) return null;

    return {
      totalContainers: stats.total_containers.toString(),
      cpuUsage: formatPercent(stats.total_cpu_usage_percent),
      memoryUsage: formatBytes(stats.total_memory_usage_bytes),
      memoryLimit: formatBytes(stats.total_memory_limit_bytes),
      memoryPercent: formatPercent(stats.total_memory_usage_percent),
      networkRx: formatBytes(stats.total_network_rx_bytes),
      networkTx: formatBytes(stats.total_network_tx_bytes),
      blockRead: formatBytes(stats.total_block_read_bytes),
      blockWrite: formatBytes(stats.total_block_write_bytes),
    };
  }, [stats]);

  if (!stats || !formattedStats) {
    return (
      <div className="mb-8">
        <div className="p-5 text-center text-gray-600">Loading total stats...</div>
      </div>
    );
  }

  return (
    <div className="mb-8">
      <h2 className="text-2xl mb-5 text-gray-900">Total Statistics</h2>
      <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-5">
        <StatCard title="Total Containers" value={formattedStats.totalContainers} />
        <StatCard title="CPU Usage" value={formattedStats.cpuUsage} />
        <StatCard
          title="Memory Usage"
          value={formattedStats.memoryUsage}
          unit={`/ ${formattedStats.memoryLimit}`}
        />
        <StatCard title="Memory %" value={formattedStats.memoryPercent} />
        <StatCard title="Network RX" value={formattedStats.networkRx} />
        <StatCard title="Network TX" value={formattedStats.networkTx} />
        <StatCard title="Block Read" value={formattedStats.blockRead} />
        <StatCard title="Block Write" value={formattedStats.blockWrite} />
      </div>
    </div>
  );
};

export default TotalStatsView;

