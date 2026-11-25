"""Upgrade an existing AdaBarprint.db, retaining a verified SQLite backup."""
import argparse
import hashlib
import json
import pathlib
import sqlite3
from datetime import datetime

MANIFEST = pathlib.Path(__file__).resolve().parents[1] / 'src-tauri/resources/print-schema-renames.json'
TABLES = json.loads(MANIFEST.read_text(encoding='utf-8'))
OBJECTS = (
    ('INDEX', 'ABPrintRuleCode', 'TCNMLabelPrintRuleCode'),
    ('INDEX', 'ABPrintHistoryDue', 'TCNTLabelQueueHistoryDue'),
    ('TRIGGER', 'ABPrintHistoryFinished', 'TCNTLabelQueueHistoryFinished'),
)


def quote(identifier):
    return '"' + identifier.replace('"', '""') + '"'


def table_names(db):
    return {row[0] for row in db.execute("SELECT name FROM sqlite_master WHERE type='table'")}


def rename_schema(db):
    existing = table_names(db)
    for entry in TABLES:
        old, new = entry['old'], entry['new']
        if old in existing:
            db.execute(f'ALTER TABLE {quote(old)} RENAME TO {quote(new)}')
            for field, replacement in entry['columns'].items():
                db.execute(f'ALTER TABLE {quote(new)} RENAME COLUMN {quote(field)} TO {quote(replacement)}')
    for kind, old, new in OBJECTS:
        row = db.execute('SELECT sql FROM sqlite_master WHERE name=?', (old,)).fetchone()
        if row:
            db.execute(f'DROP {kind} {quote(old)}')
            db.execute(row[0].replace(old, new, 1))


def contents(db):
    aliases = {entry['old']: entry['new'] for entry in TABLES}
    result = {}
    for table in table_names(db):
        hashes = []
        for row in db.execute('SELECT * FROM ' + quote(table)):
            if table == 'sqlite_sequence':
                row = (aliases.get(row[0], row[0]), row[1])
            hashes.append(hashlib.sha256(repr(row).encode()).digest())
        result[aliases.get(table, table)] = sorted(hashes)
    return result


def upgrade(path):
    path = path.resolve(strict=True)
    if path.name.lower() != 'adabarprint.db':
        raise ValueError('Expected AdaBarprint.db')
    with sqlite3.connect(path.as_uri() + '?mode=rw', uri=True, timeout=10) as db:
        db.execute('BEGIN IMMEDIATE')
        if not table_names(db).intersection(entry['old'] for entry in TABLES):
            print('Schema already upgraded')
            return
        before = contents(db)
        directory = path.parent / 'Backups'
        directory.mkdir(exist_ok=True)
        backup = directory / ('AdaBarprint-before-rename-' + datetime.now().strftime('%Y%m%d-%H%M%S-%f') + '.db')
        with sqlite3.connect(path.as_uri() + '?mode=ro', uri=True) as source, sqlite3.connect(backup) as snapshot:
            source.backup(snapshot)
            if snapshot.execute('PRAGMA integrity_check').fetchall() != [('ok',)] or contents(snapshot) != before:
                raise ValueError('Backup verification failed')
        rename_schema(db)
        if contents(db) != before or db.execute('PRAGMA integrity_check').fetchall() != [('ok',)] or db.execute('PRAGMA foreign_key_check').fetchone():
            raise ValueError('Schema migration verification failed')
        print('Verified', len(before), 'tables including SQLite metadata. Backup:', backup)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('path', type=pathlib.Path)
    upgrade(parser.parse_args().path)
