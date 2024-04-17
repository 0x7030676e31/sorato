/// <reference types="vite/client" />


declare global {
  interface Window {
    getIsHead: () => boolean;
    getClients: () => Client[];
    getAudio: () => Audio[];
    getGroups: () => Group[];
    getActors: () => Actor[];
  }

  type Activity = { ["Online" | "Offline"]: number };

  interface Client {}

  interface Audio {}

  interface Group {}

  interface Actor {
    id: number;
    token: string,
    name: string,
    has_access: boolean,
    activity: Activity,
  }
}

export {}