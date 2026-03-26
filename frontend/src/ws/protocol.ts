export type WsStatus = 'disconnected' | 'connecting' | 'connected' | 'error';

export type ClientToServer =
  | {
      type: 'dm';
      to: string;
      body: string;
    }
  | {
      type: 'join_room';
      room: string;
    }
  | {
      type: 'room_msg';
      room: string;
      body: string;
    };

export type ServerToClient =
  | {
      type: 'dm';
      from: string;
      body: string;
    }
  | {
      type: 'room_msg';
      room: string;
      from: string;
      body: string;
    }
  | {
      type: 'system';
      message: string;
    };

