import type { AlertEnvelope, PairingCode, Tokens } from "./types";

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

/**
 * Client for the coordination backend (packages/proto/openapi.yaml).
 *
 * Holds the access and refresh tokens, attaches the bearer header, and refreshes
 * the access token once on a 401 before retrying. The fetch function is injected
 * so it can be unit tested without a network.
 */
export class ApiClient {
  private tokens: Tokens | null = null;

  constructor(
    private baseUrl: string,
    private fetchFn: typeof fetch = fetch,
  ) {}

  get isAuthenticated(): boolean {
    return this.tokens !== null;
  }

  setTokens(tokens: Tokens | null): void {
    this.tokens = tokens;
  }

  async register(email: string, password: string): Promise<void> {
    this.tokens = await this.json<Tokens>("POST", "/v1/auth/register", { email, password });
  }

  async login(email: string, password: string): Promise<void> {
    this.tokens = await this.json<Tokens>("POST", "/v1/auth/token", {
      grantType: "password",
      email,
      password,
    });
  }

  logout(): void {
    this.tokens = null;
  }

  async createChild(): Promise<string> {
    const r = await this.authed<{ childId: string }>("POST", "/v1/family/children");
    return r.childId;
  }

  async createPairingCode(childId: string): Promise<PairingCode> {
    return this.authed<PairingCode>("POST", "/v1/pairing/codes", { childId });
  }

  async setPolicy(childDeviceId: string, policy: unknown): Promise<void> {
    await this.authedRaw("PUT", `/v1/devices/${encodeURIComponent(childDeviceId)}/policy`, policy);
  }

  async streamAlerts(wait = false): Promise<AlertEnvelope[]> {
    const path = `/v1/alerts/stream${wait ? "?wait=1" : ""}`;
    return this.authed<AlertEnvelope[]>("GET", path);
  }

  // --- internals ---------------------------------------------------------

  private async json<T>(method: string, path: string, body?: unknown): Promise<T> {
    const resp = await this.fetchFn(this.baseUrl + path, {
      method,
      headers: { "Content-Type": "application/json" },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    return this.parse<T>(resp);
  }

  private async authed<T>(method: string, path: string, body?: unknown): Promise<T> {
    return this.parse<T>(await this.authedRaw(method, path, body));
  }

  private async authedRaw(method: string, path: string, body?: unknown): Promise<Response> {
    let resp = await this.send(method, path, body);
    if (resp.status === 401 && (await this.refresh())) {
      resp = await this.send(method, path, body);
    }
    return resp;
  }

  private send(method: string, path: string, body?: unknown): Promise<Response> {
    const headers: Record<string, string> = { "Content-Type": "application/json" };
    if (this.tokens) headers.Authorization = `Bearer ${this.tokens.accessToken}`;
    return this.fetchFn(this.baseUrl + path, {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  }

  private async refresh(): Promise<boolean> {
    if (!this.tokens?.refreshToken) return false;
    const resp = await this.fetchFn(this.baseUrl + "/v1/auth/token", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ grantType: "refresh", refreshToken: this.tokens.refreshToken }),
    });
    if (!resp.ok) {
      this.tokens = null;
      return false;
    }
    const data = (await resp.json()) as { accessToken: string };
    this.tokens = { accessToken: data.accessToken, refreshToken: this.tokens.refreshToken };
    return true;
  }

  private async parse<T>(resp: Response): Promise<T> {
    const text = await resp.text();
    if (!resp.ok) {
      let message = text;
      try {
        message = (JSON.parse(text) as { error?: string }).error ?? text;
      } catch {
        /* keep raw text */
      }
      throw new ApiError(resp.status, message);
    }
    return (text ? JSON.parse(text) : undefined) as T;
  }
}
