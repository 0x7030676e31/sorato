export const base_url = import.meta.env.DEV ? "http://localhost:8080" : window.location.origin + "/sorato"

export function authorize(code: string): Promise<Response> {
  return fetch(base_url + "/api/v1actor/actor/authorize", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "Authorization": "Bearer 2138",
    },
    body: JSON.stringify(code)
  });
}

export async  function test() {
  const sse = await fetch(base_url + "/api/v1actor/stream", {
    method: "GET",
    headers: {
      "Content-Type": "text/event-stream",
      "Authorization": "I11jV3DsPO5lURS8AHK3ScZRBoyavUVYEjLytfdvUCP4pVDGLQQmkjVFTQSameZTa",
    },
  });

  const reader = sse.body?.getReader();
  if (!reader) {
    return;
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
      console.log(message);
    }
  }
}