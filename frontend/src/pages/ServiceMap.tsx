import { useState, useEffect, useRef } from 'react';
import type { ServiceMap, ServiceNode, ServiceEdge, ConnectionType } from '../types';
import { fetchServiceMap, fetchContainers } from '../services/api';
import type { ContainerInfo } from '../types';
import { Network, Server, Database } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '../components/ui/card';
import { Badge } from '../components/ui/badge';
import { Button } from '../components/ui/button';

const ServiceMap = () => {
  const [serviceMap, setServiceMap] = useState<ServiceMap | null>(null);
  const [containers, setContainers] = useState<ContainerInfo[]>([]);
  const [selectedServiceId, setSelectedServiceId] = useState<string>('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedNode, setSelectedNode] = useState<ServiceNode | null>(null);
  const [selectedEdge, setSelectedEdge] = useState<ServiceEdge | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  // Load containers list for dropdown
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

  // Load service map (filtered by selected service if any)
  useEffect(() => {
    const loadServiceMap = async () => {
      try {
        setLoading(true);
        const data = await fetchServiceMap(selectedServiceId || undefined);
        setServiceMap(data);
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Failed to load service map');
      } finally {
        setLoading(false);
      }
    };

    loadServiceMap();
  }, [selectedServiceId]);

  const getServiceIcon = (imageFamily: string) => {
    const family = imageFamily.toLowerCase();
    if (family.includes('postgres') || family.includes('mysql') || family.includes('redis') || family.includes('mongo')) {
      return Database;
    }
    if (family.includes('nginx') || family.includes('apache') || family.includes('traefik')) {
      return Network;
    }
    return Server;
  };

  const getConnectionTypeColor = (type: ConnectionType): string => {
    switch (type) {
      case 'environment_variable':
        return '#10b981'; // green
      case 'same_network':
        return '#3b82f6'; // blue
      case 'port_mapping':
        return '#8b5cf6'; // purple
      case 'network_traffic':
        return '#f59e0b'; // amber
      case 'image_based':
        return '#6b7280'; // gray
      default:
        return '#9ca3af';
    }
  };

  const getConnectionTypeLabel = (type: ConnectionType): string => {
    switch (type) {
      case 'environment_variable':
        return 'Env Var';
      case 'same_network':
        return 'Network';
      case 'port_mapping':
        return 'Port';
      case 'network_traffic':
        return 'Traffic';
      case 'image_based':
        return 'Image';
      default:
        return 'Unknown';
    }
  };

  const getStatusVariant = (status: string): 'success' | 'destructive' | 'warning' | 'secondary' => {
    const statusLower = status.toLowerCase();
    if (statusLower.includes('running') || statusLower.includes('up')) return 'success';
    if (statusLower.includes('exited') || statusLower.includes('stopped')) return 'destructive';
    if (statusLower.includes('created') || statusLower.includes('paused')) return 'warning';
    return 'secondary';
  };

  // Simple force-directed layout
  const calculateLayout = () => {
    if (!serviceMap) return { nodes: [], edges: [] };

    const nodes = serviceMap.nodes.map((node, index) => {
      // Simple circular layout for now
      const angle = (index / serviceMap.nodes.length) * 2 * Math.PI;
      const radius = 200;
      const x = 400 + radius * Math.cos(angle);
      const y = 300 + radius * Math.sin(angle);
      return { ...node, x, y };
    });

    return { nodes, edges: serviceMap.edges };
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-96">
        <div className="text-gray-500">Loading service map...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-5">
        <div className="bg-red-50 text-red-800 p-4 rounded border-l-4 border-red-800">
          <strong>Error:</strong> {error}
        </div>
      </div>
    );
  }

  if (!serviceMap || serviceMap.nodes.length === 0) {
    return (
      <div className="p-6">
        <div className="mb-6">
          <div className="flex items-center justify-between mb-4">
            <div>
              <h1 className="text-2xl font-semibold text-gray-900 mb-1">Service Communication Map</h1>
              <p className="text-sm text-gray-600">
                {selectedServiceId
                  ? 'Visual representation of connections for selected service'
                  : 'Visual representation of all service connections and communication patterns'}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <select
                value={selectedServiceId}
                onChange={(e) => setSelectedServiceId(e.target.value)}
                className="px-3 py-2 border border-gray-300 rounded-md text-sm min-w-[250px] focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                <option value="">All Services</option>
                {containers.map((container) => (
                  <option key={container.id} value={container.id}>
                    {container.name} ({container.status})
                  </option>
                ))}
              </select>
              {selectedServiceId && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setSelectedServiceId('')}
                >
                  Clear Filter
                </Button>
              )}
            </div>
          </div>
        </div>
        <div className="text-center py-20">
          <Server className="h-16 w-16 text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500 text-lg">
            {selectedServiceId
              ? 'No connections found for the selected service'
              : 'No services found'}
          </p>
        </div>
      </div>
    );
  }

  const { nodes, edges } = calculateLayout();
  const nodeMap = new Map(nodes.map((n) => [n.container_id, n]));

  return (
    <div className="p-6">
      <div className="mb-6">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h1 className="text-2xl font-semibold text-gray-900 mb-1">Service Communication Map</h1>
            <p className="text-sm text-gray-600">
              {selectedServiceId
                ? 'Visual representation of connections for selected service'
                : 'Visual representation of all service connections and communication patterns'}
            </p>
          </div>
          <div className="flex items-center gap-2">
            <select
              value={selectedServiceId}
              onChange={(e) => setSelectedServiceId(e.target.value)}
              className="px-3 py-2 border border-gray-300 rounded-md text-sm min-w-[250px] focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">All Services</option>
              {containers.map((container) => (
                <option key={container.id} value={container.id}>
                  {container.name} ({container.status})
                </option>
              ))}
            </select>
            {selectedServiceId && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => setSelectedServiceId('')}
              >
                Clear Filter
              </Button>
            )}
          </div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Graph Visualization */}
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle>Service Graph</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="relative border rounded-lg bg-gray-50 overflow-auto" style={{ minHeight: '600px' }}>
                <svg
                  ref={svgRef}
                  width="100%"
                  height="600"
                  viewBox="0 0 800 600"
                  className="cursor-pointer"
                >
                  {/* Draw edges */}
                  {edges.map((edge, idx) => {
                    const fromNode = nodeMap.get(edge.from);
                    const toNode = nodeMap.get(edge.to);
                    if (!fromNode || !toNode) return null;

                    const color = getConnectionTypeColor(edge.connection_type);
                    const opacity = Math.max(0.3, edge.confidence);

                    return (
                      <line
                        key={`edge-${idx}`}
                        x1={fromNode.x}
                        y1={fromNode.y}
                        x2={toNode.x}
                        y2={toNode.y}
                        stroke={color}
                        strokeWidth={2 * edge.confidence}
                        opacity={opacity}
                        markerEnd="url(#arrowhead)"
                        className="hover:stroke-width-4 transition-all"
                        onClick={() => setSelectedEdge(edge)}
                      />
                    );
                  })}

                  {/* Arrow marker definition */}
                  <defs>
                    <marker
                      id="arrowhead"
                      markerWidth="10"
                      markerHeight="10"
                      refX="9"
                      refY="3"
                      orient="auto"
                    >
                      <polygon points="0 0, 10 3, 0 6" fill="#666" />
                    </marker>
                  </defs>

                  {/* Draw nodes */}
                  {nodes.map((node) => {
                    const Icon = getServiceIcon(node.image_family);
                    const isSelected = selectedNode?.container_id === node.container_id;

                    return (
                      <g
                        key={node.container_id}
                        transform={`translate(${node.x}, ${node.y})`}
                        className="cursor-pointer"
                        onClick={() => setSelectedNode(node)}
                      >
                        {/* Node circle */}
                        <circle
                          r="30"
                          fill={isSelected ? '#3b82f6' : '#fff'}
                          stroke={isSelected ? '#2563eb' : '#e5e7eb'}
                          strokeWidth={isSelected ? 3 : 2}
                          className="hover:stroke-blue-500 transition-all"
                        />
                        {/* Icon */}
                        <foreignObject x="-12" y="-12" width="24" height="24">
                          <div className="flex items-center justify-center h-full">
                            <Icon className="h-5 w-5 text-gray-700" />
                          </div>
                        </foreignObject>
                        {/* Node label */}
                        <text
                          x="0"
                          y="50"
                          textAnchor="middle"
                          className="text-xs font-medium fill-gray-700"
                          style={{ fontSize: '12px' }}
                        >
                          {node.container_name.length > 15
                            ? node.container_name.substring(0, 15) + '...'
                            : node.container_name}
                        </text>
                      </g>
                    );
                  })}
                </svg>
              </div>

              {/* Legend */}
              <div className="mt-4 flex flex-wrap gap-4 text-xs">
                <div className="flex items-center gap-2">
                  <div className="w-4 h-0.5 bg-green-500"></div>
                  <span>Environment Variable</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-4 h-0.5 bg-blue-500"></div>
                  <span>Same Network</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-4 h-0.5 bg-purple-500"></div>
                  <span>Port Mapping</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-4 h-0.5 bg-amber-500"></div>
                  <span>Network Traffic</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="w-4 h-0.5 bg-gray-500"></div>
                  <span>Image Based</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Details Panel */}
        <div className="space-y-4">
          {/* Selected Node Details */}
          {selectedNode && (
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Service Details</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div>
                  <label className="text-xs font-medium text-gray-500">Container Name</label>
                  <p className="text-sm font-semibold">{selectedNode.container_name}</p>
                </div>
                <div>
                  <label className="text-xs font-medium text-gray-500">Image</label>
                  <p className="text-sm text-gray-700">{selectedNode.image}</p>
                </div>
                <div>
                  <label className="text-xs font-medium text-gray-500">Image Family</label>
                  <p className="text-sm text-gray-700">{selectedNode.image_family}</p>
                </div>
                <div>
                  <label className="text-xs font-medium text-gray-500">Status</label>
                  <div className="mt-1">
                    <Badge variant={getStatusVariant(selectedNode.status)}>
                      {selectedNode.status}
                    </Badge>
                  </div>
                </div>
                {selectedNode.networks.length > 0 && (
                  <div>
                    <label className="text-xs font-medium text-gray-500">Networks</label>
                    <div className="mt-1 flex flex-wrap gap-1">
                      {selectedNode.networks.map((network) => (
                        <Badge key={network} variant="secondary" className="text-xs">
                          {network}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}
                <div>
                  <label className="text-xs font-medium text-gray-500">Container ID</label>
                  <p className="text-xs font-mono text-gray-500">{selectedNode.container_id.substring(0, 12)}...</p>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Selected Edge Details */}
          {selectedEdge && (
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Connection Details</CardTitle>
              </CardHeader>
              <CardContent className="space-y-3">
                <div>
                  <label className="text-xs font-medium text-gray-500">Type</label>
                  <div className="mt-1">
                    <Badge
                      style={{
                        backgroundColor: getConnectionTypeColor(selectedEdge.connection_type),
                        color: 'white',
                      }}
                    >
                      {getConnectionTypeLabel(selectedEdge.connection_type)}
                    </Badge>
                  </div>
                </div>
                <div>
                  <label className="text-xs font-medium text-gray-500">Confidence</label>
                  <div className="mt-1">
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div
                        className="bg-blue-600 h-2 rounded-full"
                        style={{ width: `${selectedEdge.confidence * 100}%` }}
                      ></div>
                    </div>
                    <p className="text-xs text-gray-500 mt-1">
                      {(selectedEdge.confidence * 100).toFixed(0)}%
                    </p>
                  </div>
                </div>
                {selectedEdge.evidence.length > 0 && (
                  <div>
                    <label className="text-xs font-medium text-gray-500">Evidence</label>
                    <ul className="mt-1 space-y-1">
                      {selectedEdge.evidence.map((evidence, idx) => (
                        <li key={idx} className="text-xs text-gray-600 flex items-start gap-1">
                          <span className="text-gray-400">•</span>
                          <span>{evidence}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </CardContent>
            </Card>
          )}

          {/* Statistics */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Statistics</CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-600">Total Services</span>
                <span className="font-semibold">{serviceMap.nodes.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Total Connections</span>
                <span className="font-semibold">{serviceMap.edges.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">High Confidence</span>
                <span className="font-semibold">
                  {serviceMap.edges.filter((e) => e.confidence >= 0.7).length}
                </span>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
};

export default ServiceMap;
