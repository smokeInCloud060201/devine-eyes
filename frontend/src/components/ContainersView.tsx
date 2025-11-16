import { useMemo } from 'react';
import type { ContainerInfo, ContainerStats, DataPoint } from '../types';
import ContainerCard from './ContainerCard';

interface ContainersViewProps {
  containers: ContainerInfo[];
  containerStats: ContainerStats[];
  historicalData: Map<string, DataPoint[]>;
  onSelect: (containerId: string) => void;
}

const ContainersView = ({
  containers,
  containerStats,
  historicalData,
  onSelect,
}: ContainersViewProps) => {
  const statsMap = useMemo(() => {
    const map = new Map<string, ContainerStats>();
    containerStats.forEach((stat) => {
      map.set(stat.container_id, stat);
    });
    return map;
  }, [containerStats]);

  return (
    <div className="mb-8">
      <h2 className="text-2xl mb-5 text-gray-900">Containers</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-5">
        {containers.map((container) => {
          const stats = statsMap.get(container.id) || null;
          const history = historicalData.get(container.id) || [];

          return (
            <ContainerCard
              key={container.id}
              container={container}
              stats={stats}
              historicalData={history}
              onClick={() => onSelect(container.id)}
            />
          );
        })}
      </div>
      {containers.length === 0 && (
        <div className="p-10 text-center text-gray-500 bg-white rounded-lg">
          No containers found
        </div>
      )}
    </div>
  );
};

export default ContainersView;
