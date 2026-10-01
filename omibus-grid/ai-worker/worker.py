"""Standalone supervised research worker, using only the Python standard library."""
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))

import argparse
import ctypes
import json
import os
import threading
import time
from engine import Engine, DEFAULTS, validate_config
from market import fetch


def atomic_json(path, value):
    temporary = path.with_suffix('.tmp')
    with temporary.open('w', encoding='utf-8') as file:
        json.dump(value, file, allow_nan=False)
        file.flush()
        os.fsync(file.fileno())
    os.replace(temporary, path)


def parent_watch(pid):
    if not pid or os.name != 'nt':
        return
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.OpenProcess.argtypes = [ctypes.c_ulong, ctypes.c_int, ctypes.c_ulong]
    kernel.OpenProcess.restype = ctypes.c_void_p
    kernel.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
    kernel.CloseHandle.argtypes = [ctypes.c_void_p]
    handle = kernel.OpenProcess(0x00100000, False, pid)  # SYNCHRONIZE only
    if not handle:
        os._exit(0)
    try:
        while kernel.WaitForSingleObject(handle, 5000) == 0x102:
            pass
    finally:
        kernel.CloseHandle(handle)
    os._exit(0)  # SQLite rolls back any uncommitted transaction after abrupt parent exit.


def main():
    parser = argparse.ArgumentParser(description='Omibus AI Research: public data, no exchange orders')
    parser.add_argument('--data-dir', required=True)
    parser.add_argument('--config')
    parser.add_argument('--parent-pid', type=int)
    parser.add_argument('--once', action='store_true')
    args = parser.parse_args()
    directory = Path(args.data_dir).resolve()
    directory.mkdir(parents=True, exist_ok=True)
    config = validate_config(json.loads(Path(args.config).read_text(encoding='utf-8-sig')) if args.config else dict(DEFAULTS))
    # A second desktop instance must not train or write this journal concurrently.
    lock = (directory / 'worker.lock').open('a+b')
    if os.name == 'nt':
        import msvcrt
        if lock.tell() == 0:
            lock.write(b'0'); lock.flush()
        lock.seek(0)
        try:
            msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
        except OSError:
            raise SystemExit('Another AI worker is already using this data directory. Stop it in the other app first.')
    else:
        import fcntl
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    if args.parent_pid:
        threading.Thread(target=parent_watch, args=(args.parent_pid,), daemon=True).start()
    engine = Engine(directory, config)
    status_file = directory / 'status.json'
    report = None
    try:
        while True:
            try:
                atomic_json(status_file, dict(phase='fetching', heartbeat=int(time.time()*1000), report=report, error=None))
                fetch_started = time.monotonic()
                batch, now = fetch(config['symbol'], config['interval'], config['history_bars'], engine.next_fetch())
                engine.ingest(batch, now)
                atomic_json(status_file, dict(phase='evaluating', heartbeat=int(time.time()*1000), report=report, error=None))
                # Conservatively advance exchange time by all processing time,
                # so a slow fetch or training run cannot paper-fill a past open.
                if report is None or (batch and batch[-1][0]>report['latest_candle']) or now>=report['next_train']:
                    report = engine.cycle(now, issue_time=lambda: now + int((time.monotonic()-fetch_started)*1000))
                atomic_json(status_file, dict(phase='waiting', heartbeat=int(time.time()*1000), report=report, error=None, next_poll=int(time.time()*1000)+15000))
                if args.once:
                    print(json.dumps(report, allow_nan=False))
                    return
                time.sleep(15)
            except Exception as error:
                retry_seconds = max(300, getattr(error, 'retry_seconds', 300))
                atomic_json(status_file, dict(phase='error', heartbeat=int(time.time()*1000), report=report,
                                             error=str(error), retry_at=int(time.time()*1000)+retry_seconds*1000))
                if args.once:
                    raise
                time.sleep(retry_seconds)
    finally:
        engine.close()
        lock.close()


if __name__ == '__main__':
    main()
