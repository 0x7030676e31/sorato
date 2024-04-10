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