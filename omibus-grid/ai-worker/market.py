"""Public Binance Spot OHLCV only. No credentials or order endpoints."""
import json
import math
import time
import urllib.error
import urllib.parse
import urllib.request
from inputs import CATALOG

BASE = 'https://data-api.binance.vision'
INTERVALS = {item['id']:item['ms'] for item in CATALOG['intervals']}

def alignment_offset(step):
    return 4*86400000 if step == 604800000 else 0  # Binance weeks start on Monday.


class RateLimited(RuntimeError):
    def __init__(self, retry_seconds):
        super().__init__(f'Binance rate limited this IP; next attempt in {retry_seconds} seconds.')
        self.retry_seconds = retry_seconds


def request(endpoint, params=None):
    url = BASE + endpoint + ('?' + urllib.parse.urlencode(params) if params else '')
    for attempt in range(3):
        try:
            req = urllib.request.Request(url, headers={'User-Agent': 'Omibus-AI-Research/1.0'})
            with urllib.request.urlopen(req, timeout=20) as response:
                return json.load(response)
        except urllib.error.HTTPError as error:
            if error.code in (418, 429):
                try:
                    retry_seconds = max(300, int(error.headers.get('Retry-After', '300')))
                except (ValueError, TypeError):
                    retry_seconds = 300
                raise RateLimited(retry_seconds) from error
            if error.code < 500 or attempt == 2:
                raise RuntimeError(f'Binance public data unavailable (HTTP {error.code}). Check symbol, network and regional availability.') from error
        except (urllib.error.URLError, TimeoutError, OSError):
            if attempt == 2:
                raise RuntimeError('Cannot reach Binance public data. Check the internet connection.')
        time.sleep(2 ** attempt)


def validate(candles, step, now_ms):
    previous = None
    for row in candles:
        if len(row) != 7:
            raise ValueError('Invalid candle shape')
        stamp, o, h, l, c, volume, close_time = row
        if not all(math.isfinite(x) for x in row) or min(o, h, l, c) <= 0 or volume < 0:
            raise ValueError('Non-finite, negative or zero candle data')
        if l > min(o, c) or h < max(o, c) or l > h:
            raise ValueError('Invalid OHLC range')
        if (stamp-alignment_offset(step)) % step != 0 or close_time != stamp + step - 1 or close_time >= now_ms:
            raise ValueError('Unclosed or misaligned candle')
        if previous is not None and stamp != previous + step:
            raise ValueError('Gap or duplicate in candles; refusing to train across missing history')
        previous = stamp
    return candles


def fetch(symbol, interval, bars, start=None):
    now_ms = int(request('/api/v3/time')['serverTime'])
    step = INTERVALS[interval]
    end = now_ms - (now_ms-alignment_offset(step)) % step - 1
    cursor = max(0,start if start is not None else end + 1 - bars * step)
    candles = []
    while cursor <= end:
        raw = request('/api/v3/klines', {'symbol': symbol, 'interval': interval, 'startTime': cursor, 'endTime': end, 'limit': 1000})
        if not raw:
            break
        batch = [[int(r[0]), *[float(r[k]) for k in range(1, 6)], int(r[6])] for r in raw if int(r[6]) <= end]
        if not batch or batch[-1][0] < cursor:
            break
        candles.extend(batch)
        cursor = batch[-1][0] + step
        if len(candles) > 50000:
            raise RuntimeError('Catch-up exceeds 50,000 candles. Start a new research profile.')
    return validate(candles, step, now_ms), now_ms
