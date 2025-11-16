import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  ContainerInfo,
  ContainerStats,
  TotalStats,
  ContainerLog,
  DataPoint,
} from '../types';
import {
  fetchContainers,
  fetchAllContainerStats,
  fetchContainerLogs,
  connectSSEStats,
} from '../services/api';
import TotalStatsView from '../components/TotalStatsView';
import ContainersView from '../components/ContainersView';
import LogsView from '../components/LogsView';

const MAX_HISTORY = 60; // Keep last 60 data points

let loaded = false;

const DockerMonitor = () => {
  const [totalStats, setTotalStats] = useState<TotalStats | null>(null);
  const [containers, setContainers] = useState<ContainerInfo[]>([]);
  const [containerStats, setContainerStats] = useState<ContainerStats[]>([]);
  const [selectedContainer, setSelectedContainer] = useState<string | null>(null);
  const [logs, setLogs] = useState<ContainerLog[]>([]);
  const [logLimit, setLogLimit] = useState(100);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [sseConnected, setSseConnected] = useState(false);
  const [historicalData, setHistoricalData] = useState<Map<string, DataPoint[]>>(
    new Map()
  );

  const sseRef = useRef<EventSource | null>(null);
  const intervalRef = useRef<number | null>(null);

  // Update historical data for charts - append new points smoothly
  const updateHistoricalData = useCallback((stats: ContainerStats[]) => {
    setHistoricalData((prev) => {
      const newData = new Map(prev);
      let hasChanges = false;

      stats.forEach((stat) => {
        const timestamp = new Date(stat.timestamp).getTime() / 1000;
        // Get existing entry or create new array (preserve reference if no changes)
        const existingEntry = newData.get(stat.container_id);
        let entry = existingEntry ? [...existingEntry] : [];

        // Check if this is a new timestamp (avoid duplicates)
        const isNew =
          entry.length === 0 ||
          Math.abs(entry[entry.length - 1].timestamp - timestamp) > 0.1;

        if (isNew) {
          const networkKb =
            (stat.network_rx_bytes + stat.network_tx_bytes) / 1024;

          const dataPoint: DataPoint = {
            timestamp,
            cpu: stat.cpu_usage_percent,
            memory: stat.memory_usage_percent,
            network: networkKb,
          };

          // Append new point to the end (create new array to trigger React update)
          entry = [...entry, dataPoint];
          hasChanges = true;

          // Keep only the last MAX_HISTORY points
          if (entry.length > MAX_HISTORY) {
            entry = entry.slice(-MAX_HISTORY);
          }

          newData.set(stat.container_id, entry);
        } else if (existingEntry) {
          // Preserve reference if no new data point (prevents unnecessary re-renders)
          newData.set(stat.container_id, existingEntry);
        }
      });

      // Only return new Map if there were changes, preserving references for unchanged containers
      return hasChanges ? newData : prev;
    });
  }, []);

  // Fetch containers and stats
  const refreshData = useCallback(async () => {
    try {
      if (loaded) {
        setLoading(true);
      }


      setError(null);

      // Fetch containers
      const newContainers = await fetchContainers();
      setContainers(newContainers);

      // Fetch all container stats
      const newStats = await fetchAllContainerStats();
      setContainerStats(newStats);
      updateHistoricalData(newStats);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch data');
      console.error('Error fetching data:', err);
    } finally {
      setLoading(false);
    }
  }, [updateHistoricalData]);

  // Load logs for selected container
  const loadLogs = useCallback(async () => {
    if (!selectedContainer) return;

    try {
      setLoading(true);
      setError(null);
      const containerLogs = await fetchContainerLogs(selectedContainer, logLimit);
      setLogs(containerLogs);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch logs');
      console.error('Error fetching logs:', err);
    } finally {
      setLoading(false);
    }
  }, [selectedContainer, logLimit]);

  // Set up SSE connection for total stats
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

    return () => {
      eventSource.close();
    };
  }, []);

  // Initial data fetch and periodic refresh
  useEffect(() => {
    refreshData();

    // Set up periodic refresh every 2 seconds
    intervalRef.current = window.setInterval(() => {
      refreshData();
    }, 2000);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [refreshData]);

  // Load logs when container or limit changes
  useEffect(() => {
    if (selectedContainer) {
      loadLogs();
    }
  }, [selectedContainer, logLimit, loadLogs]);

  const handleContainerSelect = useCallback((containerId: string) => {
    setSelectedContainer(containerId);
  }, []);

  const handleLogLimitChange = useCallback((limit: number) => {
    setLogLimit(limit);
  }, []);

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-semibold text-gray-900 mb-1">Dashboard</h1>
        <div className="flex items-center justify-between">
          <p className="text-sm text-gray-600">Overall System Health</p>
          <div className="flex items-center space-x-4">
            <div className={`text-xs ${sseConnected ? 'text-green-600' : 'text-red-600'}`}>
              {sseConnected ? (
                <>🟢 Connected</>
              ) : (
                <>🔴 Disconnected</>
              )}
            </div>
          </div>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 text-red-800 p-4 rounded mb-4 border-l-4 border-red-800 shadow-sm">
          <strong>Error:</strong> {error}
        </div>
      )}

      {loading && (
        <div className="p-4 text-center text-gray-600 bg-white rounded-lg mb-4 shadow-sm">
          Loading...
        </div>
      )}

      <TotalStatsView stats={totalStats} />

      <ContainersView
        containers={containers}
        containerStats={containerStats}
        historicalData={historicalData}
        onSelect={handleContainerSelect}
      />

      <LogsView
        containers={containers}
        selectedContainer={selectedContainer}
        logs={logs}
        logLimit={logLimit}
        onContainerChange={handleContainerSelect}
        onLimitChange={handleLogLimitChange}
        onLoad={loadLogs}
        onClear={() => setLogs([])}
      />
    </div>
  );
};

export default DockerMonitor;
