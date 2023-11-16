import { ApiClient } from "./api/ApiClient";

let BASE = "http://localhost:8080";
if (import.meta.env.PROD) {
  BASE = `https://${window.location.host}`;
}

export const client = new ApiClient({
  BASE,
});
