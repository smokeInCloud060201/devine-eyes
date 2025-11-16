import { useMemo } from 'react';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';
import type { DataPoint } from '../types';

interface MetricsChartProps {
  data: DataPoint[];
  width?: number;
  height?: number;
}

const MetricsChart = ({ data, width = 300, height = 120 }: MetricsChartProps) => {
  // Memoize chart data transformation to prevent unnecessary recalculations
  // Use data length as dependency to ensure we recalculate when new points are added
  const chartData = useMemo(() => {
    return data.map((point) => ({
      time: new Date(point.timestamp * 1000).toLocaleTimeString(),
      cpu: point.cpu,
      memory: point.memory,
      network: point.network,
    }));
  }, [data]);
  
  // Track if this is the initial render (no animation) or update (with animation)
  const isInitialRender = useMemo(() => data.length <= 1, [data.length]);

  if (data.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-gray-500 text-sm">
        <p>No data available</p>
      </div>
    );
  }

  return (
    <div className="bg-white rounded p-2.5" style={{ width, height }}>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={chartData} margin={{ top: 5, right: 30, left: 5, bottom: 5 }}>
          <CartesianGrid strokeDasharray="3 3" stroke="#e0e0e0" />
          <XAxis
            dataKey="time"
            stroke="#666"
            fontSize={10}
            tick={{ fill: '#666' }}
            interval="preserveStartEnd"
          />
          {/* Left Y-axis for CPU and Memory (percentages 0-100) */}
          <YAxis
            yAxisId="left"
            stroke="#666"
            fontSize={10}
            tick={{ fill: '#666' }}
            domain={[0, 100]}
            label={{ value: '%', angle: -90, position: 'insideLeft', style: { textAnchor: 'middle' } }}
          />
          {/* Right Y-axis for Network (KB/s) */}
          <YAxis
            yAxisId="right"
            orientation="right"
            stroke="#4CAF50"
            fontSize={10}
            tick={{ fill: '#4CAF50' }}
            label={{ value: 'KB/s', angle: 90, position: 'insideRight', style: { textAnchor: 'middle' } }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: 'rgba(255, 255, 255, 0.95)',
              border: '1px solid #ccc',
              borderRadius: '4px',
            }}
          />
          <Legend
            wrapperStyle={{ fontSize: '12px', paddingTop: '10px' }}
            iconType="line"
          />
          <Line
            yAxisId="left"
            type="monotone"
            dataKey="cpu"
            stroke="#FFC107"
            strokeWidth={2}
            dot={false}
            name="CPU %"
            isAnimationActive={!isInitialRender}
            animationDuration={200}
            animationEasing="ease-out"
            connectNulls={false}
            activeDot={{ r: 4 }}
          />
          <Line
            yAxisId="left"
            type="monotone"
            dataKey="memory"
            stroke="#2196F3"
            strokeWidth={2}
            dot={false}
            name="Memory %"
            isAnimationActive={!isInitialRender}
            animationDuration={200}
            animationEasing="ease-out"
            connectNulls={false}
            activeDot={{ r: 4 }}
          />
          <Line
            yAxisId="right"
            type="monotone"
            dataKey="network"
            stroke="#4CAF50"
            strokeWidth={2}
            dot={false}
            name="Network KB/s"
            isAnimationActive={!isInitialRender}
            animationDuration={200}
            animationEasing="ease-out"
            connectNulls={false}
            activeDot={{ r: 4 }}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};

export default MetricsChart;

