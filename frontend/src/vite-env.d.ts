/// <reference types="vite/client" />


declare global {
  interface Window {
    getIsHead: () => boolean;
    getActors: () => Actor[];
  }

  type Activity = { ["Online" | "Offline"]: number };

  interface Actor {
    id: number;
    token: string,
    name: string,
    has_access: boolean,
    activity: Activity,
  }
}

export {}