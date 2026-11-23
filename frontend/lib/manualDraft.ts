interface SavedDraft<T> {
  owner: string;
  savedAt: string;
  sets: T[];
}

function openDrafts(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('adabarprint-manual-drafts', 1);
    request.onupgradeneeded = () => request.result.createObjectStore('drafts', { keyPath: 'owner' });
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => reject(request.error);
  });
}

export async function readDraft<T>(owner: string): Promise<SavedDraft<T> | undefined> {
  const db = await openDrafts();
  try {
    return await new Promise<SavedDraft<T> | undefined>((resolve, reject) => {
      const request = db.transaction('drafts').objectStore('drafts').get(owner);
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
  } finally { db.close(); }
}

export async function writeDraft<T>(owner: string, sets: T[]): Promise<void> {
  const db = await openDrafts();
  try {
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction('drafts', 'readwrite');
      tx.objectStore('drafts').put({ owner, sets, savedAt: new Date().toISOString() });
      tx.oncomplete = () => resolve();
      tx.onabort = () => reject(tx.error);
    });
  } finally { db.close(); }
}
