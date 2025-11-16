import { useState, useMemo } from 'react';
import type { ContainerInfo, ContainerLog } from '../types';
import { formatDate } from '../utils/formatting';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Card, CardContent } from './ui/card';

interface LogsViewProps {
  containers: ContainerInfo[];
  selectedContainer: string | null;
  logs: ContainerLog[];
  logLimit: number;
  onContainerChange: (containerId: string) => void;
  onLimitChange: (limit: number) => void;
  onLoad: () => void;
  onClear: () => void;
}

const LogsView = ({
  containers,
  selectedContainer,
  logs,
  logLimit,
  onContainerChange,
  onLimitChange,
  onLoad,
  onClear,
}: LogsViewProps) => {
  const [limitInput, setLimitInput] = useState(logLimit.toString());
  const [searchQuery, setSearchQuery] = useState('');

  // Filter logs based on search query
  const filteredLogs = useMemo(() => {
    if (!searchQuery.trim()) {
      return logs;
    }

    const query = searchQuery.toLowerCase();
    return logs.filter((log) => log.log_line.toLowerCase().includes(query));
  }, [logs, searchQuery]);

  const handleLimitSubmit = () => {
    const limit = parseInt(limitInput, 10);
    if (!isNaN(limit) && limit > 0) {
      onLimitChange(limit);
    }
  };

  return (
    <div className="mb-8">
      <h2 className="text-2xl mb-5 text-gray-900">Container Logs</h2>
      <div className="flex gap-2.5 mb-5 flex-wrap items-center">
        <select
          value={selectedContainer || ''}
          onChange={(e) => onContainerChange(e.target.value)}
          className="px-3 py-2 border border-gray-300 rounded text-sm min-w-[250px]"
        >
          <option value="">Select a container...</option>
          {containers.map((container) => (
            <option key={container.id} value={container.id}>
              {container.name} ({container.status})
            </option>
          ))}
        </select>
        <div className="flex gap-1">
          <Input
            type="number"
            placeholder="Limit"
            min="1"
            max="10000"
            value={limitInput}
            onChange={(e) => setLimitInput(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleLimitSubmit()}
            className="w-24"
          />
          <Button onClick={handleLimitSubmit} variant="outline" size="sm">
            Set Limit
          </Button>
        </div>
        <Button onClick={onLoad} disabled={!selectedContainer} size="sm">
          Load Logs
        </Button>
        <Button onClick={onClear} variant="outline" size="sm">
          Clear
        </Button>
      </div>

      {/* Search/Filter Input */}
      {logs.length > 0 && (
        <div className="mb-5">
          <div className="relative">
            <Input
              type="text"
              placeholder="Search logs... (case-insensitive)"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full pl-10"
            />
            <svg
              className="absolute left-3 top-2.5 h-5 w-5 text-gray-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              />
            </svg>
            {searchQuery && (
              <button
                onClick={() => setSearchQuery('')}
                className="absolute right-3 top-2.5 text-gray-400 hover:text-gray-600"
                aria-label="Clear search"
              >
                <svg className="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M6 18L18 6M6 6l12 12"
                  />
                </svg>
              </button>
            )}
          </div>
          {searchQuery && (
            <div className="mt-2 text-sm text-gray-600">
              Showing {filteredLogs.length} of {logs.length} log entries
            </div>
          )}
        </div>
      )}

      <Card className="max-h-[600px] overflow-y-auto">
        <CardContent className="p-5">
        {filteredLogs.length > 0 ? (
          <div className="font-mono text-sm">
            {filteredLogs.map((log, index) => (
              <div
                key={`${log.container_id}-${log.timestamp}-${index}`}
                className={`py-1 border-b border-gray-100 flex gap-2.5 last:border-b-0 ${
                  log.stream === 'stderr' ? 'log-stderr' : ''
                }`}
              >
                <span className="text-gray-600 whitespace-nowrap flex-shrink-0">
                  [{formatDate(log.timestamp)}]
                </span>
                <span
                  className={`break-words ${
                    log.stream === 'stderr' ? 'text-red-600' : 'text-gray-900'
                  }`}
                >
                  {log.log_line}
                </span>
              </div>
            ))}
          </div>
        ) : searchQuery && logs.length > 0 ? (
          <div className="p-10 text-center text-gray-500">
            No logs match your search query: &quot;{searchQuery}&quot;
          </div>
        ) : selectedContainer ? (
          <div className="p-10 text-center text-gray-500">No logs available for this container.</div>
        ) : (
          <div className="p-10 text-center text-gray-500">Select a container to view logs...</div>
        )}
        </CardContent>
      </Card>
    </div>
  );
};

export default LogsView;
