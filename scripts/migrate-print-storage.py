"""One-time migration: copy print-owned data, never copy product master tables.

Run with the desktop, agent and development backend stopped. No print or sync
operations are performed. A consistent SQLite backup is kept before migration.
Existing targets are never overwritten. Source retirement is a separate step.
"""
import argparse
import hashlib
import json
import pathlib
import sqlite3
from datetime import datetime

TABLES = (
    'TCNMPrinterServer', 'TCNSSysOption', 'TCNMPaperSize', 'TCNMPreprint',
    'TCNMLabelField', 'TCNMLabelSize', 'TCNMLabelTmpl', 'TCNMLabelPrinter',
    'TCNMPdtLabel', 'TCNTLabelJob', 'TCNTLabelJobDt', 'TCNTLabelLog',
    'TCNTLabelSchedule', 'TCNTLabelSchedRun', 'TCNTLabelSchedQueue',
    'TCNTLabelSchedWatermark', 'ABPrintRule', 'ABPrintSetting', 'ABPrintJob',
    'ABPrintUnit', 'ABPrintWatermark', 'ABPrintAudit',
)
RETIRED_OPTIONS = ('MasterApiUrl', 'MasterApiKey', 'MasterAgnCode', 'MasterBchCode', 'MasterPosCode', 'AutoPrintRetryWaitSec')

def quote(s):
    return '"' + s.replace('"', '""') + '"'

def digest(rows):
    # repr supports bytes; sort makes verification independent of query order.
    return hashlib.sha256('\n'.join(sorted(map(repr, rows))).encode()).hexdigest()

def migrate(source, target):
    source, target = source.resolve(strict=True), target.resolve()
    if source == target or target.exists():
        raise ValueError('Target must be a new file; existing databases are not overwritten')
    target.parent.mkdir(parents=True, exist_ok=True)
    backup_dir = target.parent.parent / 'Backups'
    backup_dir.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now().strftime('%Y%m%d-%H%M%S-%f')
    backup = backup_dir / f'LegacyMaster-{stamp}.sqlite.bak'
    src = sqlite3.connect(source.as_uri() + '?mode=ro', uri=True)
    snap = sqlite3.connect(backup)
    src.backup(snap)
    src.close()
    # Archive is self-contained; it must not require WAL/SHM sidecars to read.
    snap.execute('PRAGMA journal_mode=DELETE')
    stage = target.with_suffix('.migration.tmp')
    if stage.exists():
        raise ValueError('Migration staging file already exists; inspect it before retrying')
    dst = sqlite3.connect(stage)
    # ATTACH requires the same encoding as installed StoreFront (UTF-16LE).
    dst.execute("PRAGMA encoding='UTF-16le'")
    report = {}
    try:
        dst.execute('BEGIN IMMEDIATE')
        for table in TABLES:
            ddl = snap.execute("SELECT sql FROM sqlite_master WHERE type='table' AND name=?", (table,)).fetchone()
            if not ddl:
                continue
            dst.execute(ddl[0])
            rows = snap.execute('SELECT * FROM ' + quote(table)).fetchall()
            if rows:
                dst.executemany('INSERT INTO ' + quote(table) + ' VALUES (' + ','.join('?' for _ in rows[0]) + ')', rows)
            copied = dst.execute('SELECT * FROM ' + quote(table)).fetchall()
            if digest(rows) != digest(copied):
                raise ValueError(f'Copy verification failed: {table}')
            report[table] = {'rows': len(rows), 'sha256': digest(rows)}
            for (index,) in snap.execute("SELECT sql FROM sqlite_master WHERE type='index' AND tbl_name=? AND sql IS NOT NULL", (table,)):
                dst.execute(index)
        if 'TCNSSysOption' in report:
            dst.executemany('DELETE FROM TCNSSysOption WHERE FTSopCode=?', [(x,) for x in RETIRED_OPTIONS])
        if 'ABPrintSetting' in report:
            dst.execute("DELETE FROM ABPrintSetting WHERE id LIKE 'sync%' OR id='heartbeat'")
        if 'ABPrintRule' in report:
            for key, raw in dst.execute('SELECT id,config FROM ABPrintRule').fetchall():
                rule = json.loads(raw)
                rule['enabled'] = False
                dst.execute('UPDATE ABPrintRule SET config=? WHERE id=?', (json.dumps(rule, ensure_ascii=False), key))
        if 'TCNTLabelSchedule' in report:
            dst.execute("UPDATE TCNTLabelSchedule SET FTLscStaUse='2', FTLscSyncMode='STOREFRONT'")
        if dst.execute('PRAGMA integrity_check').fetchone()[0] != 'ok':
            raise ValueError('Target integrity check failed')
        dst.commit()
    except Exception:
        dst.rollback()
        raise
    finally:
        dst.close()
        snap.close()
    stage.rename(target)
    result = {'source': str(source), 'target': str(target), 'backup': str(backup),
              'verifiedBeforeSanitizing': report,
              'changes': ['Removed retired API settings and sync status', 'Disabled schedules pending Master setup', 'Sessions not copied'],
              'masterTablesCopied': False}
    report_path = backup_dir / f'Migration-{stamp}.json'
    report_path.write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding='utf-8')
    print(json.dumps({'target': str(target), 'backup': str(backup), 'report': str(report_path), 'tables': len(report)}, ensure_ascii=False))

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=pathlib.Path, required=True)
    parser.add_argument('--target', type=pathlib.Path, required=True)
    args = parser.parse_args()
    migrate(args.source, args.target)
