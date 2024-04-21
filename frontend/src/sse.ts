import { Payload } from "./api";

export default class ServerSentEvents {
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