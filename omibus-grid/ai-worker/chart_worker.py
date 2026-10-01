"""Read-only, one-shot chart snapshot. Never opens research databases or trains."""
import sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
import json
import re
from market import request, validate, INTERVALS
from inputs import normalize_inputs
from patterns import scan_patterns
from indicators import panels, indicator_series
from triggers import validate_triggers, eligibility


def chart_snapshot(config,candles,now):
    if not re.fullmatch(r'[A-Z0-9]{5,20}',config['symbol']) or config['interval'] not in INTERVALS: raise ValueError('Invalid chart market')
    selected=normalize_inputs(config['inputs'])
    config={**config,'inputs':selected,'triggers':validate_triggers(config.get('triggers',{'buy':[],'sell':[]}),selected)}
    step=INTERVALS[config['interval']]
    closed=[r for r in candles if r[6]<now]
    validate(closed,step,now)
    if len(closed)<101: raise ValueError('This market/timeframe needs at least 101 completed candles for the research chart and indicator warmup.')
    if len(candles)>650: raise ValueError('Invalid chart history length')
    if len(candles)>len(closed):
        # Validate the provisional candle's shape/alignment without classifying it as closed.
        validate(candles,step,candles[-1][6]+1)
        if candles[-1][0]>now or len(candles)!=len(closed)+1: raise ValueError('Invalid provisional candle')
    _,confirmed,forming=scan_patterns(closed,selected,include_forming=True)
    series=indicator_series(closed,selected)
    custom=[]
    for i in range(max(100,len(closed)-240),len(closed)):
        passed=eligibility(config,series,i)
        for side in ('buy','sell'):
            # For level conditions show only the transition to true; crossings are already pulses.
            previous=eligibility(config,series,i-1)[side] if i>100 else False
            if config['triggers'][side] and passed[side] and not previous:
                custom.append(dict(time=closed[i][0],price=closed[i][4],kind='RULE '+side.upper()))
    return dict(config=config,updated_at=now,provisional=len(candles)>len(closed),candles=candles,
                panels=panels(candles,selected),confirmed=confirmed[-60:],forming=forming,
                custom_triggers=custom,rule_status=eligibility(config,series,len(closed)-1),
                delayed=now-candles[-1][0]>2*step)


def main():
    config=json.load(sys.stdin)
    if set(config)!={'symbol','interval','inputs','triggers'}: raise ValueError('Invalid chart request')
    if not re.fullmatch(r'[A-Z0-9]{5,20}',config['symbol']) or config['interval'] not in INTERVALS: raise ValueError('Invalid chart market')
    config['inputs']=normalize_inputs(config['inputs'])
    config['triggers']=validate_triggers(config['triggers'],config['inputs'])
    raw=request('/api/v3/klines',{'symbol':config['symbol'],'interval':config['interval'],'limit':600})
    now=int(request('/api/v3/time')['serverTime'])
    data=[[int(r[0]),*[float(r[k]) for k in range(1,6)],int(r[6])] for r in raw]
    print(json.dumps(chart_snapshot(config,data,now),allow_nan=False))


if __name__=='__main__':
    try: main()
    except Exception as error:
        print(json.dumps({'error':str(error),'retry_seconds':max(15,getattr(error,'retry_seconds',15))}))
