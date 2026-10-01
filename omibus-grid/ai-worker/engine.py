"""Version 3 research: selected inputs, causal formations and custom entry filters."""
import hashlib
import json
import re
import sqlite3
from pathlib import Path

from features import feature_rows, WARMUP
from inputs import BY_ID, DEFAULT_INPUTS, normalize_inputs, feature_names
from patterns import scan_patterns
from market import INTERVALS, validate
from model import DirectionalModel, direction, directional_return, target_price, directional_metrics, trade_metrics
from indicators import indicator_series
from triggers import validate_triggers, eligibility

DEFAULTS = dict(symbol='BTCUSDT', interval='1h', horizon=4, history_bars=3000,
                fee_bps=10.0, slippage_bps=5.0, threshold=0.6, take_profit_pct=1.0,
                inputs=list(DEFAULT_INPUTS), triggers={'buy':[], 'sell':[]})
SCHEMA = 3
TRAIN_PERIOD = 24*3600*1000
MIN_COMPARISONS = 100


def validate_config(config):
    if set(config) != set(DEFAULTS):
        raise ValueError('Unknown or missing AI settings; select inputs and restart this profile')
    if not isinstance(config['symbol'], str) or not re.fullmatch(r'[A-Z0-9]{5,20}', config['symbol']):
        raise ValueError('Use a Binance symbol such as BTCUSDT')
    if config['interval'] not in INTERVALS:
        raise ValueError('Unsupported candle interval')
    for name, low, high in [('horizon',1,24), ('history_bars',1500,10000)]:
        if type(config[name]) is not int or not low <= config[name] <= high:
            raise ValueError(f'{name} must be between {low} and {high}')
    for name, low, high in [('fee_bps',0,100), ('slippage_bps',0,100), ('threshold',0.5,0.95), ('take_profit_pct',0.1,50)]:
        if type(config[name]) not in (float,int) or not low <= config[name] <= high:
            raise ValueError(f'Invalid {name}')
    selected = normalize_inputs(config['inputs'])
    return {**config, 'inputs': selected, 'triggers': validate_triggers(config['triggers'],selected)}


def profile_id(config):
    data = dict(schema=SCHEMA, config=validate_config(config))
    return hashlib.sha256(json.dumps(data,sort_keys=True).encode()).hexdigest()[:16]


def outcome(candles, entry_index, config, side, partial=False):
    """TP at a conservative limit fill, or time exit. All supplied bars are closed.
    Intrabar TP time is unknown: use that bar's close as its accounting timestamp.
    A gap beyond TP still fills at the target. No intrabar stop-loss is assumed.
    """
    entry = candles[entry_index][1]
    target = target_price(entry,config['take_profit_pct'],config['fee_bps'],config['slippage_bps'],side)
    end = entry_index+config['horizon']-1
    for i in range(entry_index,min(end+1,len(candles))):
        row = candles[i]
        if (side == 'LONG' and row[2] >= target) or (side == 'SHORT' and row[3] <= target):
            return dict(side=side,entry_price=entry,exit_price=target,exit_time=row[6],reason='TP',
                        net=directional_return(entry,target,config['fee_bps'],config['slippage_bps'],side))
    if end >= len(candles):
        if partial:
            return None
        raise ValueError('Outcome is not mature')
    row = candles[end]
    return dict(side=side,entry_price=entry,exit_price=row[4],exit_time=row[6],reason='TIME EXIT',
                net=directional_return(entry,row[4],config['fee_bps'],config['slippage_bps'],side))


def example(candles,i,config,allowed=None):
    long, short = outcome(candles,i+2,config,'LONG'), outcome(candles,i+2,config,'SHORT')
    return dict(stamp=candles[i][0], entry_time=candles[i+2][0], exit_time=candles[i+1+config['horizon']][6],
                long_outcome=long,short_outcome=short,y=int(long['net'] > 0),short_y=int(short['net'] > 0),net=long['net'],
                long_allowed=allowed['buy'] if allowed else True,short_allowed=allowed['sell'] if allowed else True)


def allowed_rows(candles,config):
    if not any(config['triggers'].values()): return [{'buy':True,'sell':True}]*len(candles)
    series=indicator_series(candles,config['inputs'])
    return [eligibility(config,series,i) for i in range(len(candles))]


def independent(records):
    result,last_exit = [],-1
    for row in sorted(records,key=lambda r:r['entry_time']):
        if row['entry_time'] > last_exit:
            result.append(row)
            last_exit = row['exit_time']
    return result


def evaluate(model,examples,xs,config):
    rows = []
    for i,row in examples:
        p,short_p = model.predict(xs[i])
        rows.append({**row,'p':p,'short_p':short_p})
    return directional_metrics(rows,threshold=config['threshold'])


def bootstrap(candles,xs,config,now):
    valid = list(range(WARMUP,len(candles)-config['horizon']-1))
    if len(valid) < 800:
        raise ValueError(f'Training needs at least 800 labeled examples after warmup; only {len(valid)} are available for this market/timeframe. Choose a shorter timeframe or a market with more history. The live chart remains available.')
    a,b = valid[int(len(valid)*0.6)],valid[int(len(valid)*0.8)]
    allowed=allowed_rows(candles,config)
    samples = [(i,example(candles,i,config,allowed[i])) for i in valid]
    train = [(i,r) for i,r in samples if i < a and r['exit_time'] < candles[a][0]]
    validation = [(i,r) for i,r in samples if a <= i < b and r['exit_time'] < candles[b][0]]
    test = [(i,r) for i,r in samples if i >= b]
    model = DirectionalModel(feature_names(config['inputs']))
    for i,row in train:
        model.learn(xs[i],row)
    baseline = [sum(r[key] for _,r in train)/len(train) for key in ('y','short_y')]
    report = dict(train_samples=len(train),validation_samples=len(validation),test_samples=len(test),
        selected_features=[BY_ID[key]['label'] for key in config['inputs']],
        validation=evaluate(model,validation,xs,config),holdout=evaluate(model,test,xs,config),
        holdout_baseline=directional_metrics([{**r,'p':baseline[0],'short_p':baseline[1]} for _,r in test],threshold=config['threshold']),
        holdout_start=candles[b][0],holdout_end=test[-1][1]['exit_time'],
        note='60/20/20 chronological split with label purging. Only explicitly selected inputs. No automatic feature substitution.')
    learner = model.clone()
    for i,row in samples:
        if i > train[-1][0]:
            learner.learn(xs[i],row)
    return dict(schema=SCHEMA,config=config,learner=learner.dump(),champion=model.dump(),challenger=learner.dump(),
        generation=1,champion_version=1,baseline=baseline,trained_until=candles[valid[-1]][0],
        trained_outcome_until=samples[-1][1]['exit_time'],last_train=now,next_train=now+TRAIN_PERIOD,created=now,
        last_decision='Provisional selected-input model; collecting fresh forward evidence.',bootstrap=report)


def promotion_decision(rows,threshold):
    usable = independent(rows)
    if len(usable) < MIN_COMPARISONS:
        return None
    result = {name:{key:directional_metrics(block,key,threshold) for key in ('p','challenger_p','baseline_p')}
              for name,block in [('validation',usable[:50]),('test',usable[50:100])]}
    passed = all(part['challenger_p']['brier'] < min(part['p']['brier'],part['baseline_p']['brier'])
                 and part['challenger_p']['trades'] >= 5
                 and part['challenger_p']['return_pct'] > max(0,part['p']['return_pct'])
                 and part['challenger_p']['drawdown_pct'] <= part['p']['drawdown_pct'] for part in result.values())
    return dict(promoted=passed,metrics=result,samples=100,
        reason='Candidate passed both forward blocks.' if passed else 'Candidate did not pass both forward blocks; existing model retained.')


class Engine:
    def __init__(self,directory,config):
        self.config = validate_config(config)
        self.directory = Path(directory)/profile_id(self.config)
        self.directory.mkdir(parents=True,exist_ok=True)
        self.db = sqlite3.connect(self.directory/'research.sqlite',timeout=10)
        self.db.execute('PRAGMA journal_mode=WAL')
        self.db.execute('PRAGMA synchronous=FULL')
        self.db.executescript('''
            CREATE TABLE IF NOT EXISTS candles(stamp INTEGER PRIMARY KEY,data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS state(id INTEGER PRIMARY KEY CHECK(id=1),data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS signals(stamp INTEGER PRIMARY KEY,data TEXT NOT NULL,resolved INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS trades(stamp INTEGER PRIMARY KEY,data TEXT NOT NULL,status TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS runs(id INTEGER PRIMARY KEY,stamp INTEGER NOT NULL,data TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS versions(id INTEGER PRIMARY KEY,stamp INTEGER NOT NULL,data TEXT NOT NULL);
        ''')

    def close(self):
        self.db.close()

    def next_fetch(self):
        return self.db.execute('SELECT MAX(stamp) FROM candles').fetchone()[0]

    def ingest(self,batch,now):
        validate(batch,INTERVALS[self.config['interval']],now)
        with self.db:
            for row in batch:
                previous = self.db.execute('SELECT data FROM candles WHERE stamp=?',(row[0],)).fetchone()
                if previous and json.loads(previous[0]) != row:
                    raise ValueError('Provider revised a stored candle; use a separate research profile')
                self.db.execute('INSERT OR IGNORE INTO candles VALUES (?,?)',(row[0],json.dumps(row)))

    def advance_position(self,candles,by_stamp):
        saved = self.db.execute("SELECT stamp,data FROM trades WHERE status != 'CLOSED'").fetchone()
        if not saved:
            return None
        stamp,raw = saved
        trade = json.loads(raw)
        index = by_stamp.get(trade['entry_time'])
        if index is None:
            if trade['entry_time'] < candles[0][0]:
                raise ValueError('Open paper position exceeds retained candle window')
            return trade
        entry = candles[index][1]
        trade.update(status='OPEN',entry_price=entry,
            target_price=target_price(entry,self.config['take_profit_pct'],self.config['fee_bps'],self.config['slippage_bps'],trade['side']))
        resolved = outcome(candles,index,self.config,trade['side'],partial=True)
        if resolved:
            trade.update(resolved,status='CLOSED')
        self.db.execute('UPDATE trades SET data=?,status=? WHERE stamp=?',(json.dumps(trade),trade['status'],stamp))
        return None if resolved else trade

    def cycle(self,now,issue_time=None):
        candles = [json.loads(r[0]) for r in self.db.execute('SELECT data FROM candles ORDER BY stamp DESC LIMIT 20000')][::-1]
        if not candles:
            raise ValueError('No closed candles available')
        step = INTERVALS[self.config['interval']]
        validate(candles,step,now)
        xs = feature_rows(candles,self.config['inputs'])
        allowed=allowed_rows(candles,self.config)
        saved = self.db.execute('SELECT data FROM state WHERE id=1').fetchone()
        state = json.loads(saved[0]) if saved else bootstrap(candles,xs,self.config,now)
        if state['schema'] != SCHEMA or state['config'] != self.config:
            raise ValueError('Research state version/configuration mismatch')
        if saved and state['trained_until'] < candles[WARMUP][0]:
            raise ValueError('Untrained history exceeds the research window; use a separate profile')
        by_stamp = {r[0]:i for i,r in enumerate(candles)}
        with self.db:
            if not saved:
                self.db.execute('INSERT INTO runs(stamp,data) VALUES (?,?)',(now,json.dumps({'kind':'bootstrap','report':state['bootstrap']})))
                self.archive(now,state,'bootstrap')
            for stamp,raw in self.db.execute('SELECT stamp,data FROM signals WHERE resolved=0').fetchall():
                row = json.loads(raw)
                if row['exit_time'] >= now:
                    continue
                if stamp not in by_stamp:
                    raise ValueError('Missing history for an unresolved prediction')
                index = by_stamp[stamp]
                if index+1+self.config['horizon'] >= len(candles):
                    continue  # Provider has not supplied the closing candle yet.
                row.update(example(candles,index,self.config,allowed[index]))
                self.db.execute('UPDATE signals SET data=?,resolved=1 WHERE stamp=?',(json.dumps(row),stamp))
            position = self.advance_position(candles,by_stamp)
            learner = DirectionalModel.load(state['learner'])
            if now >= state['next_train']:
                learned = 0
                for i in range(WARMUP,len(candles)-self.config['horizon']-1):
                    if candles[i][0] > state['trained_until']:
                        row = example(candles,i,self.config,allowed[i])
                        assert row['exit_time'] < now
                        learner.learn(xs[i],row)
                        state.update(trained_until=candles[i][0],trained_outcome_until=row['exit_time'])
                        learned += 1
                state.update(learner=learner.dump(),last_train=now,next_train=now+TRAIN_PERIOD)
                self.db.execute('INSERT INTO runs(stamp,data) VALUES (?,?)',(now,json.dumps({'kind':'daily_training','examples':learned})))
                self.archive(now,state,'daily learner checkpoint')
            resolved = [json.loads(r[0]) for r in self.db.execute('SELECT data FROM signals WHERE resolved=1 ORDER BY stamp')]
            decision = promotion_decision([r for r in resolved if r['generation'] == state['generation']],self.config['threshold'])
            if decision:
                if decision['promoted']:
                    state['champion'] = state['challenger']
                    state['champion_version'] += 1
                state['last_decision'] = decision['reason']
                self.db.execute('INSERT INTO runs(stamp,data) VALUES (?,?)',(now,json.dumps({'kind':'model_review',**decision})))
                state['generation'] += 1
                state['challenger'] = learner.dump()
                self.archive(now,state,decision['reason'])
            latest = candles[-1]
            entry_time = latest[0]+2*step
            issued_at = issue_time() if issue_time else now
            if xs[-1] and latest[6] < issued_at < entry_time:
                p,short_p = DirectionalModel.load(state['champion']).predict(xs[-1])
                cp,csp = DirectionalModel.load(state['challenger']).predict(xs[-1])
                raw_recommendation = direction(p,short_p,self.config['threshold'])
                recommendation = direction(p,short_p,self.config['threshold'],allowed[-1]['buy'],allowed[-1]['sell'])
                closed = [json.loads(r[0]) for r in self.db.execute("SELECT data FROM trades WHERE status='CLOSED' ORDER BY stamp")]
                bankrupt = trade_metrics(closed)['bankrupt']
                action = 'HOLD '+position['side'] if position else 'WAIT' if bankrupt else recommendation
                row = dict(stamp=latest[0],issued_at=issued_at,entry_time=entry_time,exit_time=entry_time+self.config['horizon']*step-1,
                    generation=state['generation'],champion_version=state['champion_version'],p=p,short_p=short_p,
                    challenger_p=cp,challenger_short_p=csp,baseline_p=state['baseline'][0],baseline_short_p=state['baseline'][1],
                    signal=action,recommendation=recommendation,x=xs[-1],
                    long_allowed=allowed[-1]['buy'],short_allowed=allowed[-1]['sell'],
                    reason='Paper capital exhausted' if bankrupt else 'Position already pending/open' if position else 'Model score and entry conditions met' if action != 'WAIT' else 'Custom entry conditions blocked the model signal' if raw_recommendation != 'WAIT' else 'Neither side has a unique score above threshold')
                inserted = self.db.execute('INSERT OR IGNORE INTO signals(stamp,data) VALUES (?,?)',(row['stamp'],json.dumps(row)))
                if inserted.rowcount and action in ('BUY','SELL'):
                    position = dict(stamp=row['stamp'],issued_at=issued_at,entry_time=entry_time,deadline=row['exit_time'],
                        side='LONG' if action == 'BUY' else 'SHORT',status='PENDING',signal=action)
                    self.db.execute('INSERT INTO trades VALUES (?,?,?)',(row['stamp'],json.dumps(position),'PENDING'))
            self.db.execute('INSERT OR REPLACE INTO state VALUES (1,?)',(json.dumps(state,allow_nan=False),))
        return self.report(state,candles,resolved,now,position)

    def archive(self,now,state,reason):
        self.db.execute('INSERT INTO versions(stamp,data) VALUES (?,?)',(now,json.dumps({'reason':reason,'state':state},allow_nan=False)))

    def report(self,state,candles,resolved,now,position):
        signals = [json.loads(r[0]) for r in self.db.execute('SELECT data FROM signals ORDER BY stamp DESC LIMIT 12')]
        for row in signals:
            row.pop('x',None)
        trades = [json.loads(r[0]) for r in self.db.execute('SELECT data FROM trades ORDER BY stamp')]
        closed = [r for r in trades if r['status'] == 'CLOSED']
        forward = directional_metrics(resolved,threshold=self.config['threshold'])
        forward.update(trade_metrics(closed))
        detected = scan_patterns(candles,self.config['inputs'])[1]
        chart = candles[-180:]
        if detected:
            match = detected[-1]
            start = next(i for i,r in enumerate(candles) if r[0] == match['points'][0]['time'])
            chart = candles[max(0,start-10):min(len(candles),match['confirmed_index']+11)]
        weights = []
        for side in ('long','short'):
            model = state['learner'][side]
            weights.extend(dict(side=side.upper(),feature=n,weight=w) for n,w in zip(model['names'],model['weights']))
        weights.sort(key=lambda r:abs(r['weight']),reverse=True)
        return dict(schema=SCHEMA,updated_at=now,config=self.config,profile=str(self.directory),
            candle_count=self.db.execute('SELECT COUNT(*) FROM candles').fetchone()[0],latest_candle=candles[-1][0],
            indicators=[BY_ID[key]['label'] for key in self.config['inputs']],selected_features=state['bootstrap']['selected_features'],
            feature_names=feature_names(self.config['inputs']),feature_count=len(feature_names(self.config['inputs'])),
            training_updates=state['learner']['long']['updates'],last_train=state['last_train'],next_train=state['next_train'],
            champion_version=state['champion_version'],challenger_generation=state['generation'],
            evidence_count=len(independent([r for r in resolved if r['generation'] == state['generation']])),evidence_required=MIN_COMPARISONS,
            last_decision=state['last_decision'],bootstrap=state['bootstrap'],forward=forward,signals=signals,
            position=position,trades=list(reversed(trades[-200:])),patterns=detected[-20:][::-1],chart_candles=chart,
            weights=weights[:20],runs=[{'at':t,**json.loads(d)} for t,d in self.db.execute('SELECT stamp,data FROM runs ORDER BY id DESC LIMIT 8')])
