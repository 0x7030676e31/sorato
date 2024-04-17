import { batch, createSignal } from "solid-js";

export const base_url = import.meta.env.DEV ? "http://localhost:8080" : window.location.origin + "/sorato"
const key = "authorization";

export function setToken(token: string) {
  window.localStorage.setItem(key, token);
}

export function getToken() {
  return window.localStorage.getItem(key);
}

export function removeToken() {
  window.localStorage.removeItem(key);
}

type Payload = { payload: PayloadInner, ack: number, nonce: number | null };
type PayloadInner
  = { type: "ReadyHead", payload: { actors: Actor[] } }
  | { type: "Ready", payload: { has_access: boolean } }
  | { type: "AccessChanged", payload: boolean }
  | { type: "Ping" };
  
export const [ isHead, setIsHead ] = createSignal(false);
window.getIsHead = () => isHead();

export const [ clients, setClients ] = createSignal<Client[]>([]);
window.getClients = () => clients();

export const [ audio, setAudio ] = createSignal<Audio[]>([]);
window.getAudio = () => audio();

export const [ groups, setGroups ] = createSignal<Group[]>([]);
window.getGroups = () => groups();

export const [ actors, setActors ] = createSignal<Actor[]>([]);
window.getActors = () => actors();

class ServerSentEvents {
  private url: string;
  private token: string;
  
  private request?: Response;
  private timeout: number = 1000;

  private listener?: (payload: Payload) => void;

  constructor(url: string, token: string) {
    this.url = url;
    this.token = token;
  }

  public on_payload(listener: (payload: Payload) => void) {
    this.listener = listener;
  }

  public async connect() {
    this.timeout = 1000;

    while (true) {
      try {
        await this.connect_();
        break;
      } catch (e) {
        console.log(`Failed to connect to the server. Retrying in ${+(this.timeout / 1000).toFixed(2)}s`)
        await new Promise(resolve => setTimeout(resolve, this.timeout));

        this.timeout = Math.min(this.timeout * 1.5, 25_000);
      }
    }
  }

  private async connect_() {
    const req = fetch(this.url, {
      headers: {
        "Content-Type": "text/event-stream",
        "Authorization": this.token,
      }
    });

    this.request = await req.catch(() => {
      throw new Error("Failed to connect to the server");
    });

    if (this.request.status === 401) {
      return;
    }

    const reader = this.request!.body?.getReader();
    if (!reader) {
      throw new Error("Failed to connect to the server");
    }

    const decoder = new TextDecoder();
    let buffer = "";

    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        break;
      }

      buffer += decoder.decode(value, { stream: true });

      const messages = buffer.split("\n\n");
      buffer = messages.pop() || "";

      for (const message of messages) {
        const lines = message.split("\n");
        const data = lines.map(line => line.slice(6)).join("\n");
        this.listener?.(JSON.parse(data));
      }
    }

    this.request = undefined;
    throw new Error("Connection closed");
  }
}

export function authorize(code: string): Promise<Response> {
  return fetch(base_url + "/api/v1actor/actor/authorize", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(code)
  });
}

let sse: ServerSentEvents | null = null;
let last_ack = 0;
const nonces = new Set<number>();

export function useSse() {
  const [ loginShown, setLoginShown ] = createSignal(getToken() === null);
  const [ disabled, setDisabled ] = createSignal(false);
  const [ loading, setLoading ] = createSignal(true);

  function handle_payload(payload: Payload) {
    if (payload.payload.type !== "Ping") console.log(payload);
    if (payload.ack <= last_ack) {
      return;
    }

    last_ack = payload.ack;
    if (nonces.has(payload.nonce!)) {
      return;
    }

    const inner = payload.payload;
    switch (inner.type) {
      case "Ready":
        console.log("Hello, Sorato!");
        batch(() => {
          setLoading(false);
          setDisabled(!inner.payload.has_access); 
        });
        break;

      case "ReadyHead":
        console.log("Hello, Sorato! :D");
        batch(() => {
          setLoading(false);
          setIsHead(true);
          setActors(inner.payload.actors);
        });
        break;

      case "AccessChanged":
        batch(() => {
          setDisabled(!inner.payload);
        });
        break;
    }
  }

  async function connect() {
    if (sse) {
      return;
    }

    sse = new ServerSentEvents(base_url + "/api/v1actor/stream", getToken()!);
    sse.on_payload(handle_payload);
    
    await sse.connect();
    console.log("Token deauthorized");
    sse = null;

    removeToken();
    batch(() => {
      setLoginShown(true);
      setLoading(false);
      setDisabled(false);
    });
  }
  
  return {
    loginShown,
    setLoginShown,
    connect,
    loading,
    disabled,
  }
}
