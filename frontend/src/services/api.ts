import type {
  ContainerInfo,
  ContainerStats,
  TotalStats,
  ContainerLog,
  ImageInfo,
  ServiceMap,
  HttpRequest,
} from '../types';

const API_BASE =
  (import.meta as { env?: { VITE_API_URL?: string } }).env?.VITE_API_URL ||
  'http://127.0.0.1:8080';

async function fetchJson<T>(url: string): Promise<T> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }
  return response.json();
}

// Total Stats
export async function fetchTotalStats(): Promise<TotalStats> {
  const response = await fetchJson<{ data: TotalStats }>(
    `${API_BASE}/api/stats/total`
  );
  return response.data;
}

// Containers
export async function fetchContainers(): Promise<ContainerInfo[]> {
  return fetchJson<ContainerInfo[]>(`${API_BASE}/api/containers`);
}

// Container Stats
export async function fetchContainerStats(
  containerId: string
): Promise<ContainerStats> {
  return fetchJson<ContainerStats>(
    `${API_BASE}/api/containers/${containerId}/stats`
  );
}

export async function fetchAllContainerStats(): Promise<ContainerStats[]> {
  return fetchJson<ContainerStats[]>(`${API_BASE}/api/containers/stats`);
}

export async function fetchContainerStatsHistory(
  containerId: string,
  params?: {
    from?: string;
    to?: string;
    limit?: number;
  }
): Promise<ContainerStats[]> {
  const searchParams = new URLSearchParams();
  if (params?.from) searchParams.set('from', params.from);
  if (params?.to) searchParams.set('to', params.to);
  if (params?.limit) searchParams.set('limit', params.limit.toString());

  const url = `${API_BASE}/api/containers/${containerId}/stats/history${
    searchParams.toString() ? `?${searchParams.toString()}` : ''
  }`;
  return fetchJson<ContainerStats[]>(url);
}

// Container Logs
export async function fetchContainerLogs(
  containerId: string,
  limit: number = 100
): Promise<ContainerLog[]> {
  return fetchJson<ContainerLog[]>(
    `${API_BASE}/api/containers/${containerId}/logs?limit=${limit}`
  );
}

// Images
export async function fetchImages(): Promise<ImageInfo[]> {
  return fetchJson<ImageInfo[]>(`${API_BASE}/api/images`);
}

export async function fetchImage(imageId: string): Promise<ImageInfo> {
  return fetchJson<ImageInfo>(`${API_BASE}/api/images/${imageId}`);
}

// SSE Connection for real-time stats
export function connectSSEStats(
  onMessage: (stats: TotalStats) => void,
  onError?: (error: Event) => void
): EventSource {
  const eventSource = new EventSource(`${API_BASE}/api/stats/total/sse`);

  eventSource.onmessage = (event) => {
    try {
      const stats: TotalStats = JSON.parse(event.data);
      onMessage(stats);
    } catch (error) {
      console.error('Failed to parse SSE data:', error);
    }
  };

  eventSource.onerror = (error) => {
    console.error('SSE connection error:', error);
    if (onError) {
      onError(error);
    }
  };

  return eventSource;
}

// Service Map
export async function fetchServiceMap(serviceId?: string): Promise<ServiceMap> {
  const url = serviceId
    ? `${API_BASE}/api/services/map?service_id=${encodeURIComponent(serviceId)}`
    : `${API_BASE}/api/services/map`;
  return fetchJson<ServiceMap>(url);
}

// HTTP Requests
export async function fetchContainerHttpRequests(
  containerId: string,
  limit: number = 100
): Promise<HttpRequest[]> {
  return fetchJson<HttpRequest[]>(
    `${API_BASE}/api/containers/${containerId}/requests?limit=${limit}`
  );
}

