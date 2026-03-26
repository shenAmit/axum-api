import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import type { ClientToServer, ServerToClient, WsStatus } from './protocol';

function toWsUrl(baseUrl: string) {
  // Supports:
  // - ws://host:port
  // - wss://host:port
  // - http://host:port (converted to ws://)
  // - https://host:port (converted to wss://)
  if (baseUrl.startsWith('ws://') || baseUrl.startsWith('wss://')) return baseUrl;
  if (baseUrl.startsWith('https://')) return baseUrl.replace(/^https:\/\//, 'wss://');
  if (baseUrl.startsWith('http://')) return baseUrl.replace(/^http:\/\//, 'ws://');
  return baseUrl;
}

export function useChatSocket(args: { userId: string | null; baseUrl: string }) {
  const { userId, baseUrl } = args;

  const wsUrl = useMemo(() => {
    const root = toWsUrl(baseUrl).replace(/\/+$/, '');
    return userId ? `${root}/ws?userId=${encodeURIComponent(userId)}` : `${root}/ws`;
  }, [baseUrl, userId]);

  const [status, setStatus] = useState<WsStatus>('disconnected');
  const [events, setEvents] = useState<ServerToClient[]>([]);
  const wsRef = useRef<WebSocket | null>(null);

  const connect = useCallback(() => {
    if (!userId) return;

    setStatus('connecting');
    setEvents([]);

    const ws = new WebSocket(wsUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      setStatus('connected');
    };

    ws.onmessage = (ev) => {
      try {
        const data = JSON.parse(ev.data) as ServerToClient;
        setEvents((prev) => [...prev, data]);
      } catch {
        // Ignore malformed payloads
      }
    };

    ws.onclose = () => {
      setStatus('disconnected');
    };

    ws.onerror = () => {
      setStatus('error');
    };
  }, [userId, wsUrl]);

  const disconnect = useCallback(() => {
    wsRef.current?.close();
    wsRef.current = null;
    setStatus('disconnected');
  }, []);

  const send = useCallback(
    (msg: ClientToServer) => {
      const ws = wsRef.current;
      if (!ws || ws.readyState !== WebSocket.OPEN) return;
      ws.send(JSON.stringify(msg));
    },
    [wsRef],
  );

  useEffect(() => {
    if (!userId) return;
    connect();

    return () => disconnect();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [userId]);

  return { status, events, send };
}

