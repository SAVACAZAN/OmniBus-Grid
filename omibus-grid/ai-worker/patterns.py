"""Causal geometric formation detector, not a trained vision model.

Three bars on each side confirm a swing. A formation is emitted only on a
subsequent closed-bar neckline crossing. No event is backdated to a pivot.
Fixed tolerances are heuristics in bars/relative prices, not probability scores.
"""
from inputs import BY_ID

PATTERN_IDS = [key for key, item in BY_ID.items() if item['kind'] == 'pattern']
RADIUS = 3
MAX_WIDTH = 120
ACTIVE_BARS = 20


def geometry(points, candles, sign, is_head):
    """Transform bottoms into tops, then apply the same symmetric geometry."""
    values = [sign * p['price'] for p in points]
    first, last = points[0]['index'], points[-1]['index']
    if not 12 <= last - first <= MAX_WIDTH or first < 10:
        return None
    if is_head:
        left, low1, head, low2, right = values
        n1, n2 = points[1]['index'], points[3]['index']
        slope = (low2 - low1) / (n2 - n1)
        neck = lambda index: low1 + slope * (index - n1)
        height = head - neck(points[2]['index'])
        if height <= 0 or abs(left-right) > 0.35*height or min(head-left, head-right) < 0.2*height:
            return None
        if min(left-neck(first), right-neck(last)) < 0.25*height or abs(low2-low1) > 0.5*height:
            return None
        ratio = (points[2]['index']-first) / (last-points[2]['index'])
        if not 0.4 <= ratio <= 2.5:
            return None
        extreme = head
    else:
        left, low, right = values
        height = min(left, right)-low
        if height <= 0 or abs(left-right) > 0.2*height:
            return None
        neck = lambda index: low
        extreme = max(left, right)
    # Reject insignificant fluctuations and reversals without a preceding trend.
    if height < abs(candles[first][4])*0.005:
        return None
    if sign*candles[first-10][4] > values[0]-height*0.5:
        return None
    return neck, height, extreme


def candidates(pivots,candles,selected,index):
    from pattern_shapes import shape, extended_shape
    for name in sorted(selected):
        basic=name in ('head_shoulders','inverse_head_shoulders','double_top','double_bottom')
        count=5 if 'shoulders' in name or name.startswith('triple') else 3 if name.startswith('double') else 4 if 'cup_handle' in name else 5
        for start in range(max(0,len(pivots)-10),len(pivots)-count+1):
            points=pivots[start:start+count]
            if not RADIUS <= index-points[-1]['index'] <= 30: continue
            if basic:
                is_head='shoulders' in name
                sign=-1 if name in ('inverse_head_shoulders','double_bottom') else 1
                expected=('HLHLH' if is_head else 'HLH') if sign==1 else ('LHLHL' if is_head else 'LHL')
                if ''.join(p['kind'] for p in points)!=expected: continue
                geometry_result=geometry(points,candles,sign,is_head)
                if geometry_result is None: continue
                neck,height,extreme=geometry_result
                boundary=(sign*neck(0),sign*(neck(1)-neck(0)))
                candidate=shape(name,points,height,lower=boundary if sign==1 else None,upper=boundary if sign==-1 else None,
                    direction='bearish' if sign==1 else 'bullish',invalid_upper=extreme if sign==1 else None,invalid_lower=-extreme if sign==-1 else None)
            else:
                candidate=extended_shape(name,points,candles)
            if candidate: yield candidate


def scan_patterns(candles, selected=None, include_forming=False):
    from pattern_shapes import at
    selected=set(PATTERN_IDS if selected is None else selected)&set(PATTERN_IDS)
    rows,events,pivots,seen,active=[],[],[],set(),{}
    forming={}
    for i,candle in enumerate(candles):
        j=i-RADIUS
        if j>=RADIUS:
            high,low=candles[j][2],candles[j][3]
            neighborhood=candles[j-RADIUS:j]+candles[j+1:i+1]
            is_high=all(high>r[2] for r in neighborhood); is_low=all(low<r[3] for r in neighborhood)
            if is_high != is_low:
                pivot=dict(index=j,price=high if is_high else low,kind='H' if is_high else 'L')
                if pivots and pivots[-1]['kind']==pivot['kind']:
                    sign=1 if is_high else -1
                    if sign*pivot['price']>sign*pivots[-1]['price']: pivots[-1]=pivot
                else: pivots.append(pivot)
                pivots=[p for p in pivots if p['index']>=i-MAX_WIDTH-30]
        for candidate in candidates(pivots,candles,selected,i):
            name=candidate['id']; points=candidate['points']
            key=(name,*(p['index'] for p in points))
            if key in seen: continue
            since=candles[points[-1]['index']+1:i+1]
            if candidate['invalid_upper'] is not None and any(r[4]>candidate['invalid_upper'] for r in since): continue
            if candidate['invalid_lower'] is not None and any(r[4]<candidate['invalid_lower'] for r in since): continue
            direction=candidate['direction']; boundary=None
            if direction in ('bullish','either') and candidate['upper']:
                line=candidate['upper']
                if candles[i-1][4]<=at(line,i-1) and candle[4]>at(line,i)+candidate['height']*0.02:
                    direction='bullish'; boundary=line
            if boundary is None and direction in ('bearish','either') and candidate['lower']:
                line=candidate['lower']
                if candles[i-1][4]>=at(line,i-1) and candle[4]<at(line,i)-candidate['height']*0.02:
                    direction='bearish'; boundary=line
            if 'shoulders' in name:
                labels=['Left shoulder','Neckline','Head','Neckline','Right shoulder']
            elif 'cup_handle' in name:
                labels=['Left rim','Cup bottom','Right rim','Handle']
            else: labels=['Swing '+str(n+1) for n in range(len(points))]
            lines=[dict(label=label,points=[dict(time=candles[points[0]['index']][0],price=at(line,points[0]['index'])),dict(time=candle[0],price=at(line,i))]) for label,line in [('Resistance',candidate['upper']),('Support',candidate['lower'])] if line]
            record=dict(id=name,name=BY_ID[name]['label'],direction=direction,detected_at=candle[6],
                points=[dict(time=candles[p['index']][0],price=p['price'],label=label) for p,label in zip(points,labels)],lines=lines,
                bull_trigger=at(candidate['upper'],i)+candidate['height']*0.02 if candidate['upper'] and direction in ('bullish','either') else None,
                bear_trigger=at(candidate['lower'],i)-candidate['height']*0.02 if candidate['lower'] and direction in ('bearish','either') else None)
            if boundary is not None:
                seen.add(key); active[name]=(i,candidate)
                record.update(status='confirmed',confirmed_at=candle[6],confirmed_index=i,close=candle[4],
                    neckline=at(boundary,i),neckline_points=[dict(time=candles[points[0]['index']][0],price=at(boundary,points[0]['index'])),dict(time=candle[0],price=at(boundary,i))])
                events.append(record)
            elif i==len(candles)-1:
                # Already-outside, missed breaks are not advertised as new setups.
                inside=(not candidate['upper'] or candle[4]<=at(candidate['upper'],i)) and (not candidate['lower'] or candle[4]>=at(candidate['lower'],i))
                if inside:
                    record.update(status='forming',confirmed_at=None,confirmed_index=None)
                    if name not in forming or record['points'][-1]['time']>forming[name]['points'][-1]['time']: forming[name]=record
        values={}
        for name in sorted(selected):
            value=0.0
            if name in active:
                stamp,candidate=active[name]
                valid=(candidate['invalid_upper'] is None or candle[4]<=candidate['invalid_upper']) and (candidate['invalid_lower'] is None or candle[4]>=candidate['invalid_lower'])
                if i-stamp<ACTIVE_BARS and valid: value=1-(i-stamp)/ACTIVE_BARS
                else: del active[name]
            values['pattern_'+name]=value
        rows.append(values)
    if include_forming: return rows,events,list(forming.values())
    return rows,events
