import React, { useEffect, useMemo, useRef, useState } from 'react';
import {
  ArrowRight,
  Check,
  ChevronLeft,
  ChevronRight,
  Eye,
  EyeOff,
  KeyRound,
  Loader2,
  Lock,
  LogOut,
  Search,
  User,
  Users,
  X,
} from 'lucide-react';
import {
  fetchAuthUsers,
  fetchLoginOptions,
  loadLastCredentialType,
  loadRecentUserIds,
  loginWithCredential,
  normalizeLoginMethods,
  PIN_FALLBACK_METHOD,
  rememberLastCredentialType,
  rememberRecentUserId,
  resolveLoginCredentialType,
  resolveRecentUsers,
  unlockAuthSession,
  type AuthSessionUser,
  type AuthUserSummary,
  type LoginCredentialType,
  type LoginMethodOption,
} from '@/lib/auth';

interface LoginPageProps {
  onLoginSuccess: (user: AuthSessionUser) => void;
}

interface LockScreenProps {
  user: AuthSessionUser;
  onUnlockSuccess: (user: AuthSessionUser) => void;
  onLogout: () => void;
}

const HIGHLIGHT_TAGS = ['#AdaBarprint', '#POS6', 'Label Studio', 'Pre-Print'];
const HIGHLIGHT_BULLETS = [
  'รองรับพิมพ์ป้ายราคา ป้ายโปรโมชัน และบาร์โค้ดมาตรฐาน POS6',
  'ออกแบบและปรับเลย์เอาต์ป้ายได้เองผ่าน Design Studio',
  'ผูกเครื่องพิมพ์ Windows / LAN และเลือกถาดกระดาษแยกตามประเภทป้าย',
  'เชื่อมต่อฐานข้อมูลและสิทธิ์ผู้ใช้งานเดียวกับ AdaPos6.0StoreFront',
];

const USER_BROWSER_PAGE_SIZE = 6;

function getCredentialMeta(type: LoginCredentialType) {
  if (type === '1') {
    return {
      label: 'รหัสผ่าน',
      placeholder: 'กรอกรหัสผ่าน',
      inputMode: 'text' as const,
      autoComplete: 'current-password' as const,
    };
  }
  if (type === '3') {
    return {
      label: 'บัตร RFID',
      placeholder: 'แตะหรือสแกนบัตร RFID',
      inputMode: 'text' as const,
      autoComplete: 'off' as const,
    };
  }
  return {
    label: 'รหัส PIN',
    placeholder: 'อย่างน้อย 4 หลัก',
    inputMode: 'numeric' as const,
    autoComplete: 'off' as const,
  };
}

export function LoginPage({ onLoginSuccess }: LoginPageProps) {
  const userInputRef = useRef<HTMLInputElement | null>(null);
  const credInputRef = useRef<HTMLInputElement | null>(null);
  const formRef = useRef<HTMLFormElement | null>(null);

  const [credentialType, setCredentialType] = useState<LoginCredentialType>(
    () => loadLastCredentialType() ?? '2'
  );
  const [loginMethods, setLoginMethods] = useState<LoginMethodOption[]>([PIN_FALLBACK_METHOD]);
  const [methodsLoading, setMethodsLoading] = useState(true);

  const [users, setUsers] = useState<AuthUserSummary[]>([]);
  const [usersLoading, setUsersLoading] = useState(false);
  const [usersError, setUsersError] = useState<string | null>(null);

  const [recentUserIds, setRecentUserIds] = useState<string[]>(loadRecentUserIds);
  const [userId, setUserId] = useState('');
  const [credential, setCredential] = useState('');
  const [showCred, setShowCred] = useState(false);
  const [remember, setRemember] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [browserOpen, setBrowserOpen] = useState(false);

  const activeMethod = loginMethods.find((m) => m.type === credentialType) ?? PIN_FALLBACK_METHOD;
  const requiresUserSelection = activeMethod.requiresUserSelection;
  const credMeta = getCredentialMeta(credentialType);

  const recentUsers = useMemo(
    () => resolveRecentUsers(users, recentUserIds),
    [users, recentUserIds]
  );

  const selectedUser = useMemo(
    () =>
      users.find(
        (u) =>
          u.id === userId.trim() ||
          (u.loginName && u.loginName.toLowerCase() === userId.trim().toLowerCase())
      ) ?? null,
    [users, userId]
  );

  // Load login options on mount
  useEffect(() => {
    let cancelled = false;
    setMethodsLoading(true);
    fetchLoginOptions()
      .then((res) => {
        if (cancelled) return;
        const normalized = normalizeLoginMethods(res.methods);
        setLoginMethods(normalized);
        setCredentialType(resolveLoginCredentialType(normalized, loadLastCredentialType()));
      })
      .catch(() => {
        if (cancelled) return;
        setLoginMethods([
          { type: '2', label: 'PIN', requiresUserSelection: true, isDefault: true },
          { type: '1', label: 'Password', requiresUserSelection: true, isDefault: false },
        ]);
      })
      .finally(() => {
        if (!cancelled) setMethodsLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Load users for the active credential type
  useEffect(() => {
    let cancelled = false;
    setUsersLoading(true);
    setUsersError(null);
    fetchAuthUsers(credentialType)
      .then(async (rows) => {
        if (cancelled) return;
        setUsers(rows);
        setUserId((cur) => {
          if (cur.trim()) return cur;
          return resolveRecentUsers(rows, recentUserIds)[0]?.id ?? '';
        });
        if (rows.length === 0 && credentialType !== '3') {
          const response = await fetch('/api/storage');
          if (response.ok) {
            const source = await response.json();
            if (!cancelled && source.connected === false) {
              setUsersError(source.mode === 'STANDALONE' && !source.sync?.lastSuccess
                ? 'ยังเปิดรายชื่อผู้ใช้ไม่ได้ เพราะแหล่งข้อมูลนี้ยังไม่มีผลดาวน์โหลดสำเร็จ กรุณาตรวจที่ตั้งค่าฐานข้อมูล (IT/Admin)'
                : 'ยังเปิดฐานข้อมูลสินค้าและผู้ใช้ไม่ได้ กรุณาตรวจที่ตั้งค่าฐานข้อมูล (IT/Admin)');
            }
          }
        }
      })
      .catch(() => {
        if (cancelled) return;
        setUsers([]);
        setUsersError('อ่านรายชื่อผู้ใช้ไม่ได้ กรุณาตรวจแหล่งข้อมูลและผลการดาวน์โหลดล่าสุด');
      })
      .finally(() => {
        if (!cancelled) setUsersLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [credentialType, recentUserIds]);

  // F10 hotkey submit (matching StoreFront)
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (
        event.isComposing ||
        event.key !== 'F10' ||
        submitting ||
        methodsLoading ||
        usersLoading ||
        browserOpen ||
        (requiresUserSelection && !userId.trim()) ||
        !credential.trim()
      ) {
        return;
      }
      event.preventDefault();
      event.stopImmediatePropagation();
      formRef.current?.requestSubmit();
    };
    window.addEventListener('keydown', onKeyDown, true);
    return () => window.removeEventListener('keydown', onKeyDown, true);
  }, [submitting, methodsLoading, usersLoading, browserOpen, requiresUserSelection, userId, credential]);

  const handleSelectUser = (u: AuthUserSummary) => {
    setError(null);
    setUserId(u.id);
    setCredential('');
    setBrowserOpen(false);
    window.requestAnimationFrame(() => credInputRef.current?.focus());
  };

  const handleSelectMethod = (type: LoginCredentialType) => {
    setError(null);
    setCredentialType(type);
    setCredential('');
    setBrowserOpen(false);
    window.requestAnimationFrame(() => {
      if (type !== '3' && !userId.trim()) userInputRef.current?.focus();
      else credInputRef.current?.focus();
    });
  };

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (submitting) return;
    setError(null);
    setSubmitting(true);
    try {
      const loginUserId = requiresUserSelection ? userId.trim() : '';
      const sessionUser = await loginWithCredential({
        userId: loginUserId,
        credential,
        credentialType,
        remember,
      });
      rememberLastCredentialType(credentialType);
      setRecentUserIds(rememberRecentUserId(sessionUser.id));
      onLoginSuccess(sessionUser);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'เข้าสู่ระบบไม่สำเร็จ');
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <>
      <div className="min-h-screen flex flex-col lg:flex-row bg-slate-50 text-slate-900">
        {/* Left Showcase Panel (mirrors StoreFront login-showcase with AdaBarprint teal palette) */}
        <aside className="hidden lg:flex lg:w-1/2 relative overflow-hidden bg-gradient-to-b from-[#162f37] via-[#0c4a55] to-[#007a87] text-white flex-col justify-between p-10 xl:p-12">
          <div className="pointer-events-none absolute -top-24 -left-24 h-72 w-72 rounded-full bg-teal-300/10 blur-3xl" />
          <div className="pointer-events-none absolute bottom-10 -right-20 h-80 w-80 rounded-full bg-amber-300/10 blur-3xl" />

          <div className="relative z-10 flex flex-col flex-1">
            <div className="flex items-center gap-3.5 mb-14">
              <div className="h-11 w-11 rounded-xl bg-white/95 border border-white/20 shadow-lg p-1.5 flex items-center justify-center">
                <img src="/logo.svg" alt="AdaSoft" className="h-full w-full object-contain" />
              </div>
              <div>
                <div className="text-base font-bold tracking-tight text-white">
                  AdaBarprint <span className="text-amber-300">6.0</span>
                </div>
                <div className="text-xs text-white/55">PrintBar &amp; Label Studio Workspace</div>
              </div>
            </div>

            <div className="flex-1 flex flex-col justify-center max-w-md">
              <div className="inline-flex w-fit items-center px-3 py-1 rounded-full bg-amber-300/15 border border-amber-300/30 text-amber-300 text-[11px] font-bold tracking-wider uppercase mb-4">
                AdaPos 6.0 Integration
              </div>

              <h1 className="text-3xl font-bold leading-snug text-white mb-4">
                พื้นที่ทำงานพิมพ์ป้ายราคา &amp; บาร์โค้ด
              </h1>
              <p className="text-sm leading-relaxed text-white/70 mb-6">
                ระบบพิมพ์ป้ายราคา ป้ายโปรโมชัน และออกแบบฉลากสำหรับสาขา ออกแบบมาให้ใช้บัญชีผู้ใช้และฐานข้อมูลร่วมกับ AdaStoreFront ได้ทันที
              </p>

              <div className="flex flex-wrap gap-2 mb-6">
                {HIGHLIGHT_TAGS.map((tag) => (
                  <span
                    key={tag}
                    className="inline-flex items-center px-3 py-1 rounded-full bg-white/10 border border-white/10 text-xs font-medium text-white/85"
                  >
                    {tag}
                  </span>
                ))}
              </div>

              <div className="space-y-3">
                {HIGHLIGHT_BULLETS.map((item) => (
                  <div key={item} className="flex items-center gap-2.5 text-[13px] text-white/80">
                    <span className="h-5 w-5 rounded-full bg-amber-300/20 text-amber-300 flex items-center justify-center shrink-0">
                      <Check className="h-3 w-3" />
                    </span>
                    <span>{item}</span>
                  </div>
                ))}
              </div>
            </div>

            <div className="border-t border-white/15 pt-4 text-xs text-white/45 space-y-0.5">
              <div>AdaBarprint Enterprise Edition</div>
              <div>&copy; 2026 AdaSoft Co., Ltd. All rights reserved.</div>
            </div>
          </div>
        </aside>

        {/* Right Workspace Form (mirrors StoreFront login-workspace) */}
        <main className="flex-1 flex items-center justify-center p-6 sm:p-10 bg-white">
          <div className="w-full max-w-[420px]">
            <div className="lg:hidden flex items-center gap-2.5 mb-6">
              <div className="h-9 w-9 rounded-lg bg-[#007a87]/10 border border-[#007a87]/20 p-1.5 flex items-center justify-center">
                <img src="/logo.svg" alt="AdaSoft" className="h-full w-full object-contain" />
              </div>
              <div>
                <div className="text-sm font-bold text-[#007a87]">AdaBarprint 6.0</div>
                <div className="text-[11px] text-slate-500">เข้าสู่ระบบพิมพ์ป้ายราคา</div>
              </div>
            </div>

            <div className="mb-7">
              <h2 className="text-2xl font-bold text-slate-900 mb-1.5">เข้าสู่ระบบ</h2>
              <p className="text-xs text-slate-500 leading-relaxed">
                เข้าสู่ระบบด้วยบัญชีผู้ใช้งานเดียวกับ StoreFront เพื่อเริ่มงานพิมพ์ป้ายบนเครื่องนี้
              </p>
            </div>

            {error && (
              <div
                role="alert"
                className="mb-4 rounded-lg border border-red-200 bg-red-50 px-3.5 py-2.5 text-xs font-medium text-red-800"
              >
                {error}
              </div>
            )}

            <form ref={formRef} onSubmit={handleSubmit} className="space-y-4">
              {requiresUserSelection && (
                <div>
                  <label htmlFor="login-user-id" className="block text-xs font-semibold text-slate-700 mb-1.5">
                    รหัสผู้ใช้งาน
                  </label>
                  <div className="flex items-center rounded-lg border border-slate-300 bg-white focus-within:border-[#007a87] focus-within:ring-2 focus-within:ring-[#007a87]/15 transition">
                    <span className="pl-3 text-slate-400">
                      <User className="h-4 w-4" />
                    </span>
                    <input
                      ref={userInputRef}
                      id="login-user-id"
                      name="username"
                      type="text"
                      autoComplete="username"
                      autoCapitalize="none"
                      autoCorrect="off"
                      spellCheck={false}
                      autoFocus={!userId.trim()}
                      value={userId}
                      placeholder="กรอกรหัส User จากระบบ (เช่น 00033)"
                      onChange={(e) => {
                        setError(null);
                        setUserId(e.target.value);
                        setCredential('');
                      }}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          e.preventDefault();
                          credInputRef.current?.focus();
                        }
                      }}
                      className="flex-1 min-w-0 h-10 px-2.5 text-sm bg-transparent outline-none text-slate-900 placeholder:text-slate-400"
                    />
                    <button
                      type="button"
                      disabled={usersLoading || users.length === 0}
                      onClick={() => setBrowserOpen(true)}
                      className="mr-1.5 h-7 px-2.5 rounded-md border border-slate-200 bg-slate-50 hover:bg-teal-50 hover:border-[#007a87]/40 hover:text-[#007a87] text-[11px] font-semibold text-slate-700 disabled:opacity-50 transition cursor-pointer shrink-0"
                    >
                      {usersLoading ? 'กำลังโหลด' : 'เลือกผู้ใช้'}
                    </button>
                  </div>
                  <div className="mt-1.5 text-[11px] text-slate-500 min-h-4">
                    {selectedUser
                      ? `${selectedUser.name} · ${selectedUser.role}`
                      : userId.trim()
                        ? 'จะตรวจสอบรหัสผู้ใช้นี้เมื่อเข้าสู่ระบบ'
                        : usersError ?? (usersLoading ? 'กำลังโหลดรายชื่อผู้ใช้' : users.length === 0
                          ? `ไม่พบผู้ใช้ที่มีสิทธิ์ใช้งานด้วย ${activeMethod.label} ในขณะนี้`
                          : 'พิมพ์รหัสผู้ใช้ หรือเลือกจากรายชื่อ')}
                  </div>
                </div>
              )}

              <div>
                <label htmlFor="login-credential" className="block text-xs font-semibold text-slate-700 mb-1.5">
                  {credMeta.label}
                </label>
                <div className="flex items-center rounded-lg border border-slate-300 bg-white focus-within:border-[#007a87] focus-within:ring-2 focus-within:ring-[#007a87]/15 transition">
                  <span className="pl-3 text-slate-400">
                    <KeyRound className="h-4 w-4" />
                  </span>
                  <input
                    ref={credInputRef}
                    id="login-credential"
                    name="credential"
                    type={showCred ? 'text' : 'password'}
                    inputMode={credMeta.inputMode}
                    autoComplete={credMeta.autoComplete}
                    spellCheck={false}
                    autoFocus={!requiresUserSelection || Boolean(userId.trim())}
                    value={credential}
                    placeholder={credMeta.placeholder}
                    onChange={(e) => {
                      setError(null);
                      setCredential(e.target.value);
                    }}
                    className="flex-1 min-w-0 h-10 px-2.5 text-sm bg-transparent outline-none text-slate-900 placeholder:text-slate-400"
                  />
                  <button
                    type="button"
                    tabIndex={-1}
                    onClick={() => setShowCred((s) => !s)}
                    className="pr-3 text-slate-400 hover:text-slate-600 cursor-pointer"
                    aria-label={showCred ? 'ซ่อนรหัส' : 'แสดงรหัส'}
                  >
                    {showCred ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                  </button>
                </div>
              </div>

              <div className="flex items-center justify-between pt-0.5">
                <button
                  type="button"
                  aria-pressed={remember}
                  onClick={() => setRemember((v) => !v)}
                  className="inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer select-none"
                >
                  <span
                    className={`w-9 h-5 rounded-full p-0.5 transition-colors flex items-center ${
                      remember ? 'bg-[#007a87] justify-end' : 'bg-slate-300 justify-start'
                    }`}
                  >
                    <span className="h-4 w-4 rounded-full bg-white shadow-xs" />
                  </span>
                  <span>จดจำการเข้าสู่ระบบ</span>
                </button>
              </div>

              <button
                type="submit"
                disabled={
                  submitting ||
                  methodsLoading ||
                  usersLoading ||
                  (requiresUserSelection && !userId.trim()) ||
                  !credential.trim()
                }
                className="w-full h-11 rounded-lg bg-[#007a87] hover:bg-[#00606b] disabled:opacity-50 disabled:cursor-not-allowed text-white font-semibold text-sm shadow-xs transition flex items-center justify-center gap-2 cursor-pointer"
              >
                {submitting ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    <span>กำลังตรวจสอบ...</span>
                  </>
                ) : (
                  <>
                    <span>เข้าสู่ระบบ (F10)</span>
                    <ArrowRight className="h-4 w-4" />
                  </>
                )}
              </button>

              {loginMethods.length > 1 && (
                <div
                  className="flex items-center justify-center gap-2 pt-1 text-xs"
                  role="radiogroup"
                  aria-label="เลือกวิธีเข้าสู่ระบบ"
                >
                  {loginMethods.map((method, idx) => (
                    <React.Fragment key={method.type}>
                      {idx > 0 && <span className="text-slate-300">|</span>}
                      <button
                        type="button"
                        role="radio"
                        aria-checked={credentialType === method.type}
                        onClick={() => handleSelectMethod(method.type)}
                        className={`px-2 py-1 rounded font-medium transition cursor-pointer ${
                          credentialType === method.type
                            ? 'text-[#007a87] bg-teal-50 font-semibold'
                            : 'text-slate-500 hover:text-slate-800'
                        }`}
                      >
                        {method.label}
                      </button>
                    </React.Fragment>
                  ))}
                </div>
              )}
            </form>

            {requiresUserSelection && recentUsers.length > 0 && (
              <section className="mt-7 pt-5 border-t border-slate-200" aria-label="ผู้ใช้งานล่าสุด">
                <div className="flex items-center justify-between text-xs mb-2.5">
                  <span className="font-semibold text-slate-700">ผู้ใช้งานล่าสุด</span>
                  <span className="text-[11px] text-slate-400">{recentUsers.length} บัญชี</span>
                </div>

                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                  {recentUsers.map((u) => {
                    const isSel = userId.trim() === u.id;
                    return (
                      <button
                        key={u.id}
                        type="button"
                        onClick={() => handleSelectUser(u)}
                        className={`flex items-center gap-2.5 p-2.5 rounded-lg border text-left transition cursor-pointer ${
                          isSel
                            ? 'border-[#007a87] bg-teal-50/70'
                            : 'border-slate-200 bg-slate-50/60 hover:border-slate-300 hover:bg-slate-50'
                        }`}
                      >
                        <span className="h-8 w-8 rounded-full bg-[#007a87] text-white text-xs font-bold flex items-center justify-center shrink-0">
                          {u.name.trim().charAt(0).toUpperCase() || 'U'}
                        </span>
                        <span className="min-w-0 flex-1">
                          <div className="text-xs font-semibold text-slate-800 truncate">{u.name}</div>
                          <div className="text-[11px] text-slate-500 font-mono truncate">
                            {u.id} · {u.role}
                          </div>
                        </span>
                        {isSel && <Check className="h-3.5 w-3.5 text-[#007a87] shrink-0" />}
                      </button>
                    );
                  })}

                  <button
                    type="button"
                    onClick={() => setBrowserOpen(true)}
                    className="flex items-center gap-2.5 p-2.5 rounded-lg border border-dashed border-slate-300 bg-white hover:border-[#007a87]/50 hover:bg-teal-50/40 text-left transition cursor-pointer"
                  >
                    <span className="h-8 w-8 rounded-full bg-slate-100 text-slate-600 flex items-center justify-center shrink-0">
                      <Users className="h-4 w-4" />
                    </span>
                    <span className="min-w-0 flex-1">
                      <div className="text-xs font-semibold text-slate-800">ผู้ใช้อื่น</div>
                      <div className="text-[11px] text-slate-500">ค้นหาผู้ใช้อื่น ({users.length})</div>
                    </span>
                  </button>
                </div>
              </section>
            )}
          </div>
        </main>
      </div>

      {browserOpen && requiresUserSelection && (
        <LoginUserBrowserDialog
          users={users}
          selectedUserId={userId.trim()}
          onSelect={handleSelectUser}
          onClose={() => setBrowserOpen(false)}
        />
      )}
    </>
  );
}

function LoginUserBrowserDialog({
  users,
  selectedUserId,
  onSelect,
  onClose,
}: {
  users: AuthUserSummary[];
  selectedUserId: string;
  onSelect: (u: AuthUserSummary) => void;
  onClose: () => void;
}) {
  const searchRef = useRef<HTMLInputElement | null>(null);
  const [query, setQuery] = useState('');
  const [page, setPage] = useState(0);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return users;
    return users.filter((u) =>
      `${u.id} ${u.name} ${u.loginName} ${u.role}`.toLowerCase().includes(q)
    );
  }, [users, query]);

  const pageCount = Math.max(1, Math.ceil(filtered.length / USER_BROWSER_PAGE_SIZE));
  const safePage = Math.min(page, pageCount - 1);
  const visible = filtered.slice(
    safePage * USER_BROWSER_PAGE_SIZE,
    (safePage + 1) * USER_BROWSER_PAGE_SIZE
  );

  useEffect(() => {
    searchRef.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose]);

  useEffect(() => {
    setPage(0);
  }, [query]);

  return (
    <div
      className="fixed inset-0 z-50 bg-slate-900/50 backdrop-blur-[2px] flex items-center justify-center p-4"
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <section
        role="dialog"
        aria-modal="true"
        className="w-full max-w-lg rounded-xl bg-white border border-slate-200 shadow-2xl overflow-hidden"
      >
        <header className="px-5 py-3.5 bg-[#3d4a57] text-white flex items-center justify-between">
          <div>
            <h3 className="text-sm font-semibold">เลือกผู้ใช้งาน</h3>
            <p className="text-[11px] text-slate-300">ค้นหาจากรหัสผู้ใช้ ชื่อ หรือหน้าที่</p>
          </div>
          <button
            type="button"
            onClick={onClose}
            className="h-7 w-7 rounded-md hover:bg-white/10 flex items-center justify-center cursor-pointer"
            aria-label="ปิด (Esc)"
          >
            <X className="h-4 w-4" />
          </button>
        </header>

        <div className="p-4 space-y-3">
          <div className="relative">
            <Search className="h-4 w-4 text-slate-400 absolute left-3 top-2.5" />
            <input
              ref={searchRef}
              type="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="รหัสผู้ใช้ / ชื่อ / login..."
              autoComplete="off"
              className="w-full h-9 pl-9 pr-3 rounded-lg border border-slate-300 text-xs outline-none focus:border-[#007a87] focus:ring-2 focus:ring-[#007a87]/15"
            />
          </div>

          <div className="divide-y divide-slate-100 border border-slate-200 rounded-lg overflow-hidden max-h-72 overflow-y-auto">
            {visible.length === 0 ? (
              <div className="py-8 text-center text-xs text-slate-400">ไม่พบผู้ใช้งาน</div>
            ) : (
              visible.map((u) => {
                const isSel = u.id === selectedUserId;
                return (
                  <button
                    key={u.id}
                    type="button"
                    onClick={() => onSelect(u)}
                    className={`w-full px-3.5 py-2.5 flex items-center gap-3 text-left transition cursor-pointer ${
                      isSel ? 'bg-teal-50/80' : 'hover:bg-slate-50'
                    }`}
                  >
                    <span className="h-8 w-8 rounded-full bg-[#007a87] text-white text-xs font-bold flex items-center justify-center shrink-0">
                      {u.name.trim().charAt(0).toUpperCase() || 'U'}
                    </span>
                    <div className="min-w-0 flex-1">
                      <div className="text-xs font-semibold text-slate-900 truncate">{u.name}</div>
                      <div className="text-[11px] text-slate-500 font-mono truncate">
                        {u.id}
                        {u.loginName ? ` (${u.loginName})` : ''} · {u.role}
                      </div>
                    </div>
                    <span className="text-xs font-semibold text-[#007a87]">
                      {isSel ? <Check className="h-4 w-4" /> : 'เลือก'}
                    </span>
                  </button>
                );
              })
            )}
          </div>
        </div>

        <footer className="px-4 py-2.5 bg-slate-50 border-t border-slate-200 flex items-center justify-between text-xs text-slate-600">
          <span>ทั้งหมด {filtered.length} ผู้ใช้</span>
          <div className="flex items-center gap-1.5">
            <button
              type="button"
              disabled={safePage === 0}
              onClick={() => setPage((p) => Math.max(0, p - 1))}
              className="h-7 px-2 rounded border border-slate-200 bg-white disabled:opacity-40 cursor-pointer flex items-center"
            >
              <ChevronLeft className="h-3.5 w-3.5" />
            </button>
            <span className="font-mono text-[11px] px-1.5">
              {safePage + 1} / {pageCount}
            </span>
            <button
              type="button"
              disabled={safePage >= pageCount - 1}
              onClick={() => setPage((p) => Math.min(pageCount - 1, p + 1))}
              className="h-7 px-2 rounded border border-slate-200 bg-white disabled:opacity-40 cursor-pointer flex items-center"
            >
              <ChevronRight className="h-3.5 w-3.5" />
            </button>
          </div>
        </footer>
      </section>
    </div>
  );
}

export function LockScreen({ user, onUnlockSuccess, onLogout }: LockScreenProps) {
  const [cred, setCred] = useState('');
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const handleUnlock = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!cred.trim() || busy) return;
    setBusy(true);
    setErr(null);
    try {
      const unlocked = await unlockAuthSession(cred);
      onUnlockSuccess(unlocked);
    } catch (error) {
      setErr(error instanceof Error ? error.message : 'ปลดล็อกไม่สำเร็จ');
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="min-h-screen bg-[#1f2933] flex items-center justify-center p-4">
      <div className="w-full max-w-sm rounded-xl bg-white border border-slate-200 shadow-2xl overflow-hidden">
        <div className="bg-[#3d4a57] text-white px-5 py-4 flex items-center gap-3">
          <div className="h-10 w-10 rounded-full bg-[#007a87] flex items-center justify-center shrink-0">
            <Lock className="h-5 w-5" />
          </div>
          <div className="min-w-0">
            <div className="text-sm font-semibold truncate">ล็อกหน้าจอชั่วคราว</div>
            <div className="text-xs text-slate-300 truncate">
              {user.name} ({user.id})
            </div>
          </div>
        </div>

        <form onSubmit={handleUnlock} className="p-5 space-y-4">
          {err && (
            <div className="rounded-md border border-red-200 bg-red-50 px-3 py-2 text-xs text-red-700">
              {err}
            </div>
          )}
          <div>
            <label htmlFor="unlock-credential" className="block text-xs font-semibold text-slate-700 mb-1.5">
              กรอก PIN หรือรหัสผ่านเพื่อปลดล็อก
            </label>
            <input
              id="unlock-credential"
              type="password"
              autoFocus
              value={cred}
              onChange={(e) => {
                setErr(null);
                setCred(e.target.value);
              }}
              placeholder="กรอก PIN / รหัสผ่าน"
              className="w-full h-10 px-3 rounded-lg border border-slate-300 text-sm outline-none focus:border-[#007a87] focus:ring-2 focus:ring-[#007a87]/15"
            />
          </div>

          <div className="flex items-center gap-2">
            <button
              type="submit"
              disabled={busy || !cred.trim()}
              className="flex-1 h-10 rounded-lg bg-[#007a87] hover:bg-[#00606b] disabled:opacity-50 text-white text-xs font-semibold flex items-center justify-center gap-1.5 cursor-pointer"
            >
              {busy ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <KeyRound className="h-3.5 w-3.5" />}
              <span>ปลดล็อก</span>
            </button>
            <button
              type="button"
              onClick={onLogout}
              className="h-10 px-3 rounded-lg border border-slate-300 text-slate-700 hover:bg-slate-50 text-xs font-medium flex items-center gap-1 cursor-pointer"
            >
              <LogOut className="h-3.5 w-3.5" />
              <span>ออกจากระบบ</span>
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
