/// <reference types="vite/client" />


declare global {
  interface Window {
    getIsHead: () => boolean;
    getClientId: () => number;
    getClients: () => Client[];
    getAudio: () => Audio[];
    getTempAudio: () => Audio[];
    getGroups: () => Group[];
    getActors: () => Actor[];
    getActorsMinimal: () => ActorMinimal[];
  }

  type Activity = { ["Online" | "Offline"]: number };

  interface Client {
    id: number;
    alias: string;
    hostname: string;
    username: string;
    last_ip: string;
    versions: [number, number, number]; // Loader, Module, Client
    activity: Activity;
  }

  interface Audio {
    id: number;
    title: string;
    length: number;
    downloads: number[];
    author: number | null;
    created: number;
  }

  interface Group {
    id: number;
    name: string;
    members: number[];
  }

  interface Actor {
    id: number;
    token: string,
    name: string,
    has_access: boolean,
    activity: Activity,
  }

  interface ActorMinimal {
    id: number;
    name: string;
  }
}

export {}