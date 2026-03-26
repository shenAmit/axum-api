import { useEffect, useMemo, useState } from 'react';
import { useChatSocket } from '../ws/useChatSocket';
import type { ServerToClient } from '../ws/protocol';

type Mode = 'dm' | 'room';

export function ChatApp() {
  const [userId, setUserId] = useState('');
  const [activeUserId, setActiveUserId] = useState<string | null>(null);

  const [mode, setMode] = useState<Mode>('dm');
  const [peerId, setPeerId] = useState('bob');
  const [roomId, setRoomId] = useState('room-1');

  const wsBaseUrl = useMemo(() => {
    return (import.meta.env.VITE_WS_URL as string | undefined) ?? 'http://127.0.0.1:8005';
  }, []);

  const { status, events, send } = useChatSocket({ userId: activeUserId, baseUrl: wsBaseUrl });

  const visibleMessages = useMemo(() => {
    const uid = activeUserId;
    if (!uid) return [];

    const filtered = events.filter((e) => {
      if (mode === 'dm' && e.type === 'dm') {
        return e.from === peerId || e.from === uid;
      }
      if (mode === 'room' && e.type === 'room_msg') {
        return e.room === roomId;
      }
      return false;
    });

    return filtered;
  }, [activeUserId, events, mode, peerId, roomId]);

  // Join room on mode/room changes (after socket is connected).
  useEffect(() => {
    if (mode !== 'room') return;
    if (status !== 'connected') return;
    if (!roomId.trim()) return;
    send({ type: 'join_room', room: roomId.trim() });
  }, [mode, roomId, send, status]);

  const [draft, setDraft] = useState('');

  function sendMessage() {
    if (!activeUserId) return;
    const body = draft.trim();
    if (!body) return;

    if (mode === 'dm') {
      if (!peerId.trim()) return;
      send({ type: 'dm', to: peerId.trim(), body });
    } else {
      if (!roomId.trim()) return;
      send({ type: 'room_msg', room: roomId.trim(), body });
    }

    setDraft('');
  }

  const isConnected = status === 'connected';

  if (!activeUserId) {
    return (
      <div className="panel">
        <h2 className="title">Connect to chat</h2>
        <div className="row">
          <div className="col" style={{ flex: 1 }}>
            <div className="status">Enter your userId (example: alice)</div>
            <input
              type="text"
              value={userId}
              onChange={(e) => setUserId(e.target.value)}
              placeholder="userId"
            />
          </div>
        </div>
        <div className="row">
          <button
            className="btn"
            disabled={!userId.trim()}
            onClick={() => setActiveUserId(userId.trim())}
          >
            Connect
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="chatRoot">
      <div className="panel">
        <h2 className="title">Chat</h2>
        <div className="status">status: {status}</div>

        <div className="row" style={{ marginTop: 14 }}>
          <div className="tabs">
            <button
              className={`tab ${mode === 'dm' ? 'tabActive' : ''}`}
              onClick={() => setMode('dm')}
              disabled={!isConnected}
              type="button"
            >
              1:1
            </button>
            <button
              className={`tab ${mode === 'room' ? 'tabActive' : ''}`}
              onClick={() => setMode('room')}
              disabled={!isConnected}
              type="button"
            >
              1:many
            </button>
          </div>
        </div>

        {mode === 'dm' ? (
          <div className="col" style={{ marginTop: 12 }}>
            <div className="status">Peer userId</div>
            <input type="text" value={peerId} onChange={(e) => setPeerId(e.target.value)} />
          </div>
        ) : (
          <div className="col" style={{ marginTop: 12 }}>
            <div className="status">Room</div>
            <input type="text" value={roomId} onChange={(e) => setRoomId(e.target.value)} />
          </div>
        )}

        <div className="row">
          <button className="btn" type="button" onClick={() => setActiveUserId(null)}>
            Switch user
          </button>
        </div>
      </div>

      <div className="panel">
        <div className="messages">
          {visibleMessages.length === 0 ? (
            <div className="status">No messages yet.</div>
          ) : (
            visibleMessages.map((e, idx) => {
              const m = e as ServerToClient;

              if (m.type === 'dm') {
                const isMe = m.from === activeUserId;
                return (
                  <div key={`${m.type}-${idx}`} className={`msgRow ${isMe ? 'msgMe' : 'msgOther'}`}>
                    <div className={`bubble ${isMe ? 'bubbleMe' : ''}`}>
                      <div className="meta">
                        {isMe ? 'me' : m.from}
                      </div>
                      <div>{m.body}</div>
                    </div>
                  </div>
                );
              }

              if (m.type === 'room_msg') {
                const isMe = m.from === activeUserId;
                return (
                  <div
                    key={`${m.type}-${m.room}-${idx}`}
                    className={`msgRow ${isMe ? 'msgMe' : 'msgOther'}`}
                  >
                    <div className={`bubble ${isMe ? 'bubbleMe' : ''}`}>
                      <div className="meta">
                        {m.room} • {isMe ? 'me' : m.from}
                      </div>
                      <div>{m.body}</div>
                    </div>
                  </div>
                );
              }

              if (m.type === 'system') {
                return (
                  <div key={`${m.type}-${idx}`} className="msgRow msgOther">
                    <div className="bubble">{m.message}</div>
                  </div>
                );
              }

              return null;
            })
          )}
        </div>

        <div className="inputBar">
          <input
            type="text"
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder={mode === 'dm' ? `Message ${peerId}` : `Message ${roomId}`}
            onKeyDown={(e) => {
              if (e.key === 'Enter') sendMessage();
            }}
            disabled={!isConnected}
          />
          <button className="btn" onClick={sendMessage} disabled={!isConnected || !draft.trim()}>
            Send
          </button>
        </div>
      </div>
    </div>
  );
}

