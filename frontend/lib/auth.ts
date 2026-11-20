export type LoginCredentialType = '1' | '2' | '3';

export interface LoginMethodOption {
  type: LoginCredentialType;
  label: string;
  requiresUserSelection: boolean;
  isDefault: boolean;
}

export interface AuthUserSummary {
  id: string;
  name: string;
  loginName: string;
  role: string;
}

export interface AuthSessionUser {
  id: string;
  name: string;
  role: string;
  roles: string[];
  branchId: string;
  branchName: string;
  locked: boolean;
}

const TOKEN_STORAGE_KEY = 'adabarprint_auth_token_v1';
const RECENT_USERS_KEY = 'adabarprint_recent_users_v1';
const LAST_CRED_TYPE_KEY = 'adabarprint_last_login_cred_type_v1';
const RECENT_USER_LIMIT = 3;

export const PIN_FALLBACK_METHOD: LoginMethodOption = {
  type: '2',
  label: 'PIN',
  requiresUserSelection: true,
  isDefault: true,
};

export function getStoredAuthToken(): string {
  try {
    return (
      window.localStorage.getItem(TOKEN_STORAGE_KEY) ||
      window.sessionStorage.getItem(TOKEN_STORAGE_KEY) ||
      ''
    ).trim();
  } catch {
    return '';
  }
}

export function storeAuthToken(token: string, remember: boolean): void {
  try {
    window.localStorage.removeItem(TOKEN_STORAGE_KEY);
    window.sessionStorage.removeItem(TOKEN_STORAGE_KEY);
    if (remember) {
      window.localStorage.setItem(TOKEN_STORAGE_KEY, token);
    } else {
      window.sessionStorage.setItem(TOKEN_STORAGE_KEY, token);
    }
  } catch {
    // ignore storage errors
  }
}

export function clearStoredAuthToken(): void {
  try {
    window.localStorage.removeItem(TOKEN_STORAGE_KEY);
    window.sessionStorage.removeItem(TOKEN_STORAGE_KEY);
  } catch {
    // ignore
  }
}

export function loadLastCredentialType(): LoginCredentialType | undefined {
  try {
    const val = window.localStorage.getItem(LAST_CRED_TYPE_KEY);
    if (val === '1' || val === '2' || val === '3') return val;
    return undefined;
  } catch {
    return undefined;
  }
}

export function rememberLastCredentialType(type: LoginCredentialType): void {
  try {
    window.localStorage.setItem(LAST_CRED_TYPE_KEY, type);
  } catch {
    // ignore
  }
}

export function normalizeLoginMethods(methods: LoginMethodOption[] | null | undefined): LoginMethodOption[] {
  const seen = new Set<LoginCredentialType>();
  const out = (methods ?? []).filter((m) => {
    if ((m.type !== '1' && m.type !== '2' && m.type !== '3') || seen.has(m.type)) {
      return false;
    }
    seen.add(m.type);
    return true;
  });
  return out.length > 0 ? out : [PIN_FALLBACK_METHOD];
}

export function resolveLoginCredentialType(
  methods: LoginMethodOption[],
  preferred?: LoginCredentialType
): LoginCredentialType {
  if (preferred && methods.some((m) => m.type === preferred)) return preferred;
  return methods.find((m) => m.isDefault)?.type ?? methods[0]?.type ?? '2';
}

export function loadRecentUserIds(): string[] {
  try {
    const parsed: unknown = JSON.parse(window.localStorage.getItem(RECENT_USERS_KEY) ?? '[]');
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter((v): v is string => typeof v === 'string' && v.trim().length > 0)
      .slice(0, RECENT_USER_LIMIT);
  } catch {
    return [];
  }
}

export function rememberRecentUserId(userId: string): string[] {
  const clean = userId.trim();
  const next = [clean, ...loadRecentUserIds().filter((x) => x !== clean)]
    .filter(Boolean)
    .slice(0, RECENT_USER_LIMIT);
  try {
    window.localStorage.setItem(RECENT_USERS_KEY, JSON.stringify(next));
  } catch {
    // ignore
  }
  return next;
}

export function resolveRecentUsers(users: AuthUserSummary[], recentIds: string[]): AuthUserSummary[] {
  const byId = new Map(users.map((u) => [u.id, u]));
  const picked: AuthUserSummary[] = [];
  const pushUnique = (u: AuthUserSummary | undefined) => {
    if (u && !picked.some((x) => x.id === u.id) && picked.length < RECENT_USER_LIMIT) {
      picked.push(u);
    }
  };
  recentIds.forEach((id) => pushUnique(byId.get(id)));
  users.forEach(pushUnique);
  return picked;
}

export async function fetchLoginOptions(): Promise<{ methods: LoginMethodOption[] }> {
  const res = await fetch('/api/auth/login-options');
  if (!res.ok) throw new Error('โหลดวิธีเข้าสู่ระบบไม่สำเร็จ');
  return res.json();
}

export async function fetchAuthUsers(credentialType: LoginCredentialType): Promise<AuthUserSummary[]> {
  const res = await fetch(`/api/auth/users?credentialType=${encodeURIComponent(credentialType)}`);
  if (!res.ok) throw new Error('โหลดรายชื่อผู้ใช้งานไม่สำเร็จ');
  const data = await res.json();
  return Array.isArray(data?.users) ? data.users : [];
}

export async function loginWithCredential(params: {
  userId: string;
  credential: string;
  credentialType: LoginCredentialType;
  remember: boolean;
}): Promise<AuthSessionUser> {
  const res = await fetch('/api/auth/login', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      userId: params.userId,
      credential: params.credential,
      credentialType: params.credentialType,
    }),
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok || !data?.sessionToken || !data?.user) {
    throw new Error(data?.error || 'รหัสผู้ใช้หรือข้อมูลยืนยันตัวตนไม่ถูกต้อง');
  }
  storeAuthToken(String(data.sessionToken), params.remember);
  return data.user as AuthSessionUser;
}

export async function fetchAuthMe(): Promise<AuthSessionUser | null> {
  const token = getStoredAuthToken();
  if (!token) return null;
  const res = await fetch('/api/auth/me', {
    headers: { Authorization: `Bearer ${token}` },
  });
  if (!res.ok) {
    clearStoredAuthToken();
    return null;
  }
  const data = await res.json().catch(() => null);
  return (data?.user as AuthSessionUser) ?? null;
}

export async function lockAuthSession(): Promise<void> {
  const token = getStoredAuthToken();
  if (!token) return;
  await fetch('/api/auth/lock', {
    method: 'POST',
    headers: { Authorization: `Bearer ${token}` },
  });
}

export async function unlockAuthSession(credential: string): Promise<AuthSessionUser> {
  const token = getStoredAuthToken();
  if (!token) throw new Error('Session หมดอายุ กรุณาเข้าสู่ระบบใหม่');
  const res = await fetch('/api/auth/unlock', {
    method: 'POST',
    headers: {
      Authorization: `Bearer ${token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ credential }),
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok || !data?.user) {
    throw new Error(data?.error || 'รหัส PIN หรือรหัสผ่านไม่ถูกต้อง');
  }
  return data.user as AuthSessionUser;
}

export async function logoutAuthSession(): Promise<void> {
  const token = getStoredAuthToken();
  if (token) {
    await fetch('/api/auth/logout', {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}` },
    }).catch(() => undefined);
  }
  clearStoredAuthToken();
}
