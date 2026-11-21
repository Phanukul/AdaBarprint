import { clearStoredAuthToken, getStoredAuthToken } from './auth';

// Existing screens share fetch. Attach credentials only to this application's API,
// never to product images, remote services, or other origins.
export function installApiSession(): void {
  const originalFetch = window.fetch.bind(window);
  let adminRequest: Promise<boolean> | null = null;
  window.fetch = async (input, init) => {
    const url = new URL(input instanceof Request ? input.url : String(input), window.location.href);
    if (url.origin !== window.location.origin || !url.pathname.startsWith('/api/')) {
      return originalFetch(input, init);
    }
    const headers = new Headers(init?.headers ?? (input instanceof Request ? input.headers : undefined));
    const token = getStoredAuthToken();
    if (token && !headers.has('Authorization')) headers.set('Authorization', `Bearer ${token}`);
    const replay = input instanceof Request ? input.clone() : input;
    let response = await originalFetch(input, { ...init, headers });
    if (response.status === 403) {
      const failure = await response.clone().json().catch(() => null);
      if (failure?.code === 'ADMIN_REQUIRED') {
        if (!adminRequest) {
          adminRequest = new Promise<boolean>(finish => {
            window.dispatchEvent(new CustomEvent('adabarprint-admin-required', { detail: { finish } }));
          }).finally(() => { adminRequest = null; });
        }
        if (await adminRequest) response = await originalFetch(replay, { ...init, headers });
      }
    }
    const setupOrAuth = url.pathname.startsWith('/api/auth/') || url.pathname.startsWith('/api/storage');
    if (!setupOrAuth && (response.status === 401 || response.status === 423)) {
      if (response.status === 401) clearStoredAuthToken();
      window.dispatchEvent(new Event('adabarprint-session-changed'));
    }
    return response;
  };
}
