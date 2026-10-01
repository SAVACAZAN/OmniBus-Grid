"""Small regularized online logistic model with transparent, JSON-safe state."""
import math

BASE_FEATURES = ['return_1', 'return_3', 'body', 'upper_wick', 'lower_wick', 'volume_ratio', 'range_atr']


class Model:
    def __init__(self, names, weights=None, bias=0.0, updates=0):
        self.names = list(names)
        self.weights = list(weights or [0.0] * len(names))
        self.bias, self.updates = bias, updates

    def predict(self, x):
        z = max(-30, min(30, self.bias + sum(w * x[n] for n, w in zip(self.names, self.weights))))
        return 1 / (1 + math.exp(-z))

    def learn(self, x, y):
        error = y - self.predict(x)
        rate = 0.03 / math.sqrt(1 + self.updates / 1000)
        norm = max(1.0, math.sqrt(sum(x[n] ** 2 for n in self.names)))
        self.weights = [w * (1 - rate * 0.002) + rate * error * x[n] / norm for n, w in zip(self.names, self.weights)]
        self.bias += rate * error
        self.updates += 1

    def dump(self):
        return dict(names=self.names, weights=self.weights, bias=self.bias, updates=self.updates)

    @classmethod
    def load(cls, obj):
        return cls(**obj)

    def clone(self):
        return self.load(self.dump())


class DirectionalModel:
    """Two binary learners: profitable LONG and profitable SHORT after costs."""
    def __init__(self, names, long=None, short=None):
        self.long = Model.load(long) if long else Model(names)
        self.short = Model.load(short) if short else Model(names)

    def predict(self, x):
        return self.long.predict(x), self.short.predict(x)

    def learn(self, x, example):
        self.long.learn(x, example['y'])
        self.short.learn(x, example['short_y'])

    def dump(self):
        return dict(names=self.long.names, long=self.long.dump(), short=self.short.dump())

    @classmethod
    def load(cls, state):
        return cls(**state)

    def clone(self):
        return self.load(self.dump())


def direction(long_score, short_score, threshold, long_allowed=True, short_allowed=True):
    long_score = long_score if long_allowed else -1
    short_score = short_score if short_allowed else -1
    if long_score >= threshold and long_score > short_score:
        return 'BUY'
    if short_score >= threshold and short_score > long_score:
        return 'SELL'
    return 'WAIT'


def directional_return(entry, exit_price, fee_bps, slippage_bps, side):
    if side == 'LONG':
        return net_return(entry, exit_price, fee_bps, slippage_bps)
    fee, slip = fee_bps/10000, slippage_bps/10000
    proceeds = entry*(1-slip)
    return (proceeds*(1-fee) - exit_price*(1+slip)*(1+fee)) / proceeds


def target_price(entry, target_pct, fee_bps, slippage_bps, side):
    target, fee, slip = target_pct/100, fee_bps/10000, slippage_bps/10000
    if side == 'LONG':
        return entry*(1+slip)*(1+fee)*(1+target) / ((1-slip)*(1-fee))
    return entry*(1-slip)*(1-fee-target) / ((1+slip)*(1+fee))


def trade_metrics(trades):
    equity = peak = 1.0
    drawdown = 0.0
    wins, losses = [], []
    for trade in trades:
        if equity <= 0:
            break
        result = trade['net']
        equity = max(0.0, equity*(1+result))
        peak = max(peak, equity)
        drawdown = max(drawdown, 1-equity/peak)
        (wins if result > 0 else losses).append(result)
    count = len(wins)+len(losses)
    return dict(trades=count, return_pct=100*(equity-1), drawdown_pct=100*drawdown,
                win_rate=len(wins)/count if count else None,
                expectancy_pct=100*(sum(wins)+sum(losses))/count if count else None,
                profit_factor=sum(wins)/-sum(losses) if sum(losses) < 0 else None,
                bankrupt=equity <= 0)


def directional_metrics(records, probability='p', threshold=0.6):
    short_key = {'p':'short_p', 'challenger_p':'challenger_short_p', 'baseline_p':'baseline_short_p'}[probability]
    trades, last_exit = [], -1
    for row in records:
        signal = direction(row[probability], row[short_key], threshold, row.get('long_allowed',True),row.get('short_allowed',True))
        if signal == 'WAIT' or row['entry_time'] <= last_exit:
            continue
        result = row['long_outcome' if signal == 'BUY' else 'short_outcome']
        trades.append(result)
        last_exit = result['exit_time']
    score = sum((r[probability]-r['y'])**2+(r[short_key]-r['short_y'])**2 for r in records)/(2*len(records)) if records else None
    return dict(samples=len(records), brier=score, **trade_metrics(trades))


def net_return(entry, exit_price, fee_bps, slippage_bps):
    fee, slip = fee_bps / 10000, slippage_bps / 10000
    return exit_price * (1 - slip) * (1 - fee) / (entry * (1 + slip) * (1 + fee)) - 1


def metrics(records, probability='p', threshold=0.6):
    """Non-overlapping, full-notional paper positions. Drawdown at exits only."""
    if not records:
        return {'samples': 0, 'trades': 0, 'brier': None, 'log_loss': None, 'return_pct': 0, 'drawdown_pct': 0}
    brier = sum((r[probability] - r['y']) ** 2 for r in records) / len(records)
    log_loss = -sum(r['y'] * math.log(max(1e-9, r[probability])) + (1-r['y']) * math.log(max(1e-9, 1-r[probability])) for r in records) / len(records)
    equity = peak = 1.0
    drawdown, next_entry, wins, losses = 0.0, -1, [], []
    for row in records:
        if row[probability] < threshold or row['entry_time'] <= next_entry:
            continue
        result = row['net']
        equity *= 1 + result
        peak = max(peak, equity)
        drawdown = max(drawdown, 1 - equity / peak)
        next_entry = row['exit_time']
        (wins if result > 0 else losses).append(result)
    trades = len(wins) + len(losses)
    return dict(samples=len(records), brier=brier, log_loss=log_loss,
                accuracy=sum((r[probability] >= 0.5) == bool(r['y']) for r in records) / len(records),
                trades=trades, win_rate=len(wins)/trades if trades else None,
                average_win_pct=100*sum(wins)/len(wins) if wins else None,
                average_loss_pct=100*sum(losses)/len(losses) if losses else None,
                profit_factor=sum(wins)/-sum(losses) if sum(losses) < 0 else None,
                expectancy_pct=100*(sum(wins)+sum(losses))/trades if trades else None,
                return_pct=(equity-1)*100, drawdown_pct=drawdown*100,
                drawdown_basis='closed positions only; intratrade drawdown not modeled')
