import { batch, createSignal } from "solid-js";
import ServerSentEvents from "./sse";
import progress from "./components/progress";

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

export type Payload = { payload: PayloadInner, ack: number, nonce: number | null };
export type PayloadInner
  = { type: "ReadyHead", payload: { clients: Client[], actors: Actor[], library: Audio[] } }
  | { type: "Ready", payload: { clients: Client[], actors: ActorMinimal[], library: Audio[], has_access: boolean, id: number } }
  | { type: "AccessChanged", payload: boolean }
  | { type: "AccessRevoked" }
  | { type: "AudioCreated", payload: { author: number | null, id: number, title: string, length: number, created: number } }
  | { type: "AudioDeleted", payload: number }
  | { type: "Ping" };

export const [ isHead, setIsHead ] = createSignal(false);
export const [ actorId, setActorId ] = createSignal(-1);
export const [ clients, setClients ] = createSignal<Client[]>([]);
export const [ audio, setAudio ] = createSignal<Audio[]>([]);
export const [ tempAudio, setTempAudio ] = createSignal<Audio[]>([]);
export const [ groups, setGroups ] = createSignal<Group[]>([]);
export const [ actors, setActors ] = createSignal<Actor[]>([]);
export const [ actorsMinimal, setActorsMinimal ] = createSignal<ActorMinimal[]>([]);

window.getIsHead = () => isHead();
window.getClientId = () => actorId();
window.getClients = () => clients();
window.getAudio = () => audio();
window.getTempAudio = () => tempAudio();
window.getGroups = () => groups();
window.getActors = () => actors();
window.getActorsMinimal = () => actorsMinimal();

export const [ uploading, setUploading ] = createSignal(false);

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
          setActorId(inner.payload.id);
          setDisabled(!inner.payload.has_access);
          setAudio(inner.payload.library);
          setClients(inner.payload.clients);
          setActorsMinimal(inner.payload.actors);
        });
        break;

      case "ReadyHead":
        console.log("Hello, Sorato! :D");
        batch(() => {
          setLoading(false);
          setIsHead(true);
          setActors(inner.payload.actors);
          setActorsMinimal(inner.payload.actors.map(actor => ({ id: actor.id, name: actor.name })));
          setClients(inner.payload.clients);
          setAudio(inner.payload.library);
        });
        break;

      case "AccessChanged":
        batch(() => {
          setDisabled(!inner.payload);
        });
        break;

      case "AccessRevoked":
        removeToken();
        batch(() => {
          setLoginShown(true);
          setLoading(false);
          setDisabled(false);
        });
        break;

      case "AudioCreated":
        batch(() => {
          setAudio(a => [...a, { ...inner.payload, downloads: [] }]);
        });
        break;

      case "AudioDeleted":
        batch(() => {
          setAudio(a => a.filter(audio => audio.id !== inner.payload));
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
    sse = null;

    console.log("Token deauthorized");
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
  return new Promise<void>(resolve => {
    const title = encodeURIComponent(file.name);
    const xhr = new XMLHttpRequest();
    setUploading(true);
  
    function increment(e: ProgressEvent) {
      if (e.lengthComputable) {
        progress.set((e.loaded / e.total) * 100);
      }
    }
    
    function finish() {
      progress.complete();
      setUploading(false);
      resolve();
    }

    xhr.upload.addEventListener("progress", increment);
    xhr.addEventListener("progress", increment);
    xhr.addEventListener("load", finish);

    xhr.open("POST", `${base_url}/api/v1actor/audio/upload?title=${title}`, true);
    xhr.setRequestHeader("Authorization", getToken()!);
    xhr.send(file);
  });
}

export async function delete_audio(id: number) {
  setAudio(a => a.filter(audio => audio.id !== id));
  
  const nonce = get_nonce();
  await fetch(`${base_url}/api/v1actor/audio/${id}?nonce=${nonce}`, {
    method: "DELETE",
    headers: {
      "Authorization": getToken()!
    }
  });
}
