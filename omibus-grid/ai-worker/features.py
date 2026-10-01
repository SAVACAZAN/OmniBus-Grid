"""Causal, bounded features. Every row uses only candles through that row.

Conventions: Wilder RSI/ATR/ADX; population Bollinger deviation; rolling (not
session) VWAP; OBV is represented by a 20-bar signed-volume ratio. Warmup=100.
"""
import math
from statistics import mean, pstdev
from inputs import BY_ID, feature_names
from patterns import scan_patterns

INDICATORS = ["SMA 20", "EMA 12/26", "MACD 12/26/9", "RSI 14", "Bollinger Bands 20/2",
              "ATR 14", "Stochastic %K 14", "Williams %R 14", "CCI 20", "ROC 12",
              "Momentum 10", "ADX / DI 14", "MFI 14", "OBV flow 20", "Rolling VWAP 20",
              "Chaikin Money Flow 20", "Donchian 20", "Keltner 20/2", "Aroon 25",
              "Realized volatility 20"]
WARMUP = 100


def divide(a, b, default=0.0):
    return a / b if abs(b) > 1e-15 else default


def smooth(values, period, wilder=False):
    result, previous = [], None
    alpha = 1 / period if wilder else 2 / (period + 1)
    for i, value in enumerate(values):
        if i < period - 1:
            result.append(0.0)
            continue
        previous = mean(values[:period]) if previous is None else previous + alpha * (value - previous)
        result.append(previous)
    return result


def feature_rows(candles, inputs=None):
    if not candles:
        return []
    names = feature_names(list(BY_ID) if inputs is None else inputs)
    pattern_inputs = [key for key in (BY_ID if inputs is None else inputs) if BY_ID[key]['kind'] == 'pattern']
    patterns = scan_patterns(candles, pattern_inputs)[0] if pattern_inputs else None
    c = [row[4] for row in candles]
    h = [row[2] for row in candles]
    l = [row[3] for row in candles]
    v = [row[5] for row in candles]
    tp = [(a + b + d) / 3 for a, b, d in zip(h, l, c)]
    delta = [0.0] + [c[i] - c[i - 1] for i in range(1, len(c))]
    gains = smooth([max(x, 0) for x in delta], 14, True)
    losses = smooth([max(-x, 0) for x in delta], 14, True)
    tr = [h[0] - l[0]] + [max(h[i] - l[i], abs(h[i] - c[i - 1]), abs(l[i] - c[i - 1])) for i in range(1, len(c))]
    atr = smooth(tr, 14, True)
    e12, e26, e20 = smooth(c, 12), smooth(c, 26), smooth(c, 20)
    macd = [a - b for a, b in zip(e12, e26)]
    signal = smooth(macd, 9)
    plus, minus = [0.0], [0.0]
    for i in range(1, len(c)):
        up, down = h[i] - h[i - 1], l[i - 1] - l[i]
        plus.append(max(up, 0) if up > down else 0)
        minus.append(max(down, 0) if down > up else 0)
    pdi = [100 * divide(x, a) for x, a in zip(smooth(plus, 14, True), atr)]
    mdi = [100 * divide(x, a) for x, a in zip(smooth(minus, 14, True), atr)]
    adx = smooth([100 * divide(abs(a - b), a + b) for a, b in zip(pdi, mdi)], 14, True)
    returns = [0.0] + [math.log(c[i] / c[i - 1]) for i in range(1, len(c))]
    rows = [None] * len(c)
    for i in range(WARMUP, len(c)):
        start = i - 19
        sma, sd = mean(c[start:i + 1]), pstdev(c[start:i + 1])
        high, low = max(h[i - 13:i + 1]), min(l[i - 13:i + 1])
        rsi = 50 if gains[i] + losses[i] == 0 else 100 * divide(gains[i], gains[i] + losses[i])
        stoch = divide(c[i] - low, high - low, 0.5)
        m = mean(tp[start:i + 1])
        deviation = mean([abs(x - m) for x in tp[start:i + 1]])
        positive = sum(tp[j] * v[j] for j in range(i - 13, i + 1) if tp[j] > tp[j - 1])
        negative = sum(tp[j] * v[j] for j in range(i - 13, i + 1) if tp[j] < tp[j - 1])
        mfi = divide(positive, positive + negative, 0.5)
        volume = sum(v[start:i + 1])
        vwap = divide(sum(tp[j] * v[j] for j in range(start, i + 1)), volume, c[i])
        cmf = divide(sum(divide(2*c[j]-h[j]-l[j], h[j]-l[j]) * v[j] for j in range(start, i + 1)), volume)
        obv = divide(sum((1 if delta[j] > 0 else -1 if delta[j] < 0 else 0)*v[j] for j in range(start, i + 1)), volume)
        dh, dl = max(h[start:i + 1]), min(l[start:i + 1])
        ah = max(range(i - 24, i + 1), key=lambda j: (h[j], j))
        al = min(range(i - 24, i + 1), key=lambda j: (l[j], -j))
        o, span = candles[i][1], h[i] - l[i]
        trend = divide(e12[i] - e26[i], atr[i])
        features = {
            'sma_distance': 100 * (c[i] / sma - 1),
            'ema_spread': 100 * (e12[i] / e26[i] - 1),
            'ema_fast_distance': 100 * (c[i] / e12[i] - 1),
            'ema_slow_distance': 100 * (c[i] / e26[i] - 1),
            'macd_histogram': 100 * (macd[i] - signal[i]) / c[i],
            'rsi': (rsi - 50) / 50,
            'bollinger_z': divide(c[i] - sma, sd),
            'bollinger_width': 400 * sd / sma,
            'atr_pct': 100 * atr[i] / c[i],
            'stochastic': 2 * stoch - 1,
            'williams_r': -(1 - stoch),
            'cci': divide(tp[i] - m, 0.015 * deviation) / 100,
            'roc_12': 100 * (c[i] / c[i - 12] - 1),
            'momentum_10': 100 * (c[i] / c[i - 10] - 1),
            'adx': adx[i] / 100,
            'directional_balance': (pdi[i] - mdi[i]) / 100,
            'mfi': 2 * mfi - 1, 'obv_flow': obv,
            'vwap_distance': 100 * (c[i] / vwap - 1), 'cmf': cmf,
            'donchian_position': 2 * divide(c[i] - dl, dh - dl, 0.5) - 1,
            'keltner_position': divide(c[i] - e20[i], 2 * atr[i]),
            'aroon_balance': (ah - al) / 25,
            'realized_volatility': 100 * pstdev(returns[start:i + 1]),
            'return_1': returns[i] * 100, 'return_3': 100 * (c[i] / c[i - 3] - 1),
            'body': divide(c[i] - o, span),
            'upper_wick': divide(h[i] - max(o, c[i]), span),
            'lower_wick': divide(min(o, c[i]) - l[i], span),
            'volume_ratio': divide(v[i], mean(v[start:i + 1]), 1) - 1,
            'range_atr': divide(span, atr[i]),
            'range_pct': 100 * span / c[i],
            'inside_bar': float(h[i] <= h[i - 1] and l[i] >= l[i - 1]),
            'engulfing': float(c[i] > o and c[i - 1] < candles[i - 1][1] and o <= c[i - 1] and c[i] >= candles[i - 1][1]) - float(c[i] < o and c[i - 1] > candles[i - 1][1] and o >= c[i - 1] and c[i] <= candles[i - 1][1]),
            'trend_rsi': trend * (rsi - 50) / 50,
            'trend_volume': trend * (divide(v[i], mean(v[start:i + 1]), 1) - 1),
        }
        if patterns:
            features.update(patterns[i])
        rows[i] = {key: max(-5.0, min(5.0, features[key])) for key in names}
    return rows
