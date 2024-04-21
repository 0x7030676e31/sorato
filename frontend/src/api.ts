import { batch, createSignal } from "solid-js";
import { reset, setValue } from "./components/progress";

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

export const [ tempAudio, setTempAudio ] = createSignal<Audio[]>([]);
window.getTempAudio = () => tempAudio();

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
  private disconnect?: () => void;

  constructor(url: string, token: string) {
    this.url = url;
    this.token = token;
  }

  public on_payload(listener: (payload: Payload) => void) {
    this.listener = listener;
  }

  public on_disconnect(disconnect: () => void) {
    this.disconnect = disconnect;
  }

  public async connect() {
    this.timeout = 1000;

    while (true) {
      try {
        await this.connect_();
        break;
      } catch (e) {
        console.log(`Failed to connect to the server. Retrying in ${+(this.timeout / 1000).toFixed(2)}s`);
        if (this.request) {
          this.request.body?.cancel();
          this.request = undefined;
          this.disconnect?.();
        }

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

    this.timeout = 1000;
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

function get_nonce() {
  const nonce = Math.floor(Date.now() * Math.random());
  setTimeout(() => nonces.delete(nonce), 60_000);
  nonces.add(nonce);

  return nonce;
}

let sse: ServerSentEvents | null = null;
const ack = new Set<number>();
const nonces = new Set<number>();

export function useSse() {
  const [ loginShown, setLoginShown ] = createSignal(getToken() === null);
  const [ disabled, setDisabled ] = createSignal(false);
  const [ loading, setLoading ] = createSignal(true);

  function handle_payload(payload: Payload) {
    if (payload.payload.type !== "Ping") console.log(payload);
    if (ack.has(payload.ack)) {
      return;
    }

    ack.add(payload.ack);
    setTimeout(() => ack.delete(payload.ack), 60_000);

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
    sse.on_disconnect(() => {
      setLoading(true);
    });

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

export function upload_audio(file: File) {
  return new Promise<void>((resolve, reject) => {
    const title = encodeURIComponent(file.name);
    const nonce = get_nonce();
    
    setValue(1);
    let value = 1;
    const interval = setInterval(() => {
      if (value < 100) {
        setValue(value);
        return;
      }

      clearInterval(interval);
      reset();
    }, 500);

    function progress(e: ProgressEvent) {
      if (e.lengthComputable) {
        value = (e.loaded / e.total) * 100;
      }
    }

    const xhr = new XMLHttpRequest();

    xhr.upload.addEventListener("progress", progress);
    xhr.addEventListener("progress", progress);

    xhr.onload = () => {
      // reset();
      if (xhr.status === 200) {
        resolve();
      } else {
        reject();
      }
    }

    xhr.open("POST", `${base_url}/api/v1actor/audio/upload?title=${title}&nonce=${nonce}`, true);
    xhr.setRequestHeader("Authorization", getToken()!);
    xhr.send(file);
  });
}