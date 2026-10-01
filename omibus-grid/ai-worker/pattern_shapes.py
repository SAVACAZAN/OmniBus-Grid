"""Explicit bar-based geometric rules for the extended formation catalog."""
from statistics import mean


def at(line, index):
    return line[0]+line[1]*index


def fit(points):
    xs=[p['index'] for p in points]; ys=[p['price'] for p in points]
    mx,my=mean(xs),mean(ys)
    slope=sum((x-mx)*(y-my) for x,y in zip(xs,ys))/sum((x-mx)**2 for x in xs)
    return my-slope*mx,slope


def shape(name,points,height,upper=None,lower=None,direction='either',invalid_upper=None,invalid_lower=None):
    return dict(id=name,points=points,height=height,upper=upper,lower=lower,direction=direction,
                invalid_upper=invalid_upper,invalid_lower=invalid_lower)


def extended_shape(name,points,candles):
    first,last=points[0]['index'],points[-1]['index']; span=last-first
    if first < 12 or not 12 <= span <= 120:
        return None
    prices=[p['price'] for p in points]
    kinds=''.join(p['kind'] for p in points)
    if name in ('triple_top','triple_bottom'):
        sign=1 if name=='triple_top' else -1
        if kinds != ('HLHLH' if sign==1 else 'LHLHL'): return None
        v=[sign*x for x in prices]; height=min(v[::2])-mean(v[1::2])
        if height <= abs(prices[0])*0.005 or max(v[::2])-min(v[::2]) > height*0.2 or abs(v[1]-v[3]) > height*0.2: return None
        if sign*candles[first-10][4] > v[0]-height*0.5: return None
        boundary=sign*min(v[1::2]),0
        return shape(name,points,height,lower=boundary if sign==1 else None,upper=boundary if sign==-1 else None,
                     direction='bearish' if sign==1 else 'bullish',invalid_upper=max(prices) if sign==1 else None,invalid_lower=min(prices) if sign==-1 else None)
    if name in ('cup_handle','inverse_cup_handle'):
        sign=1 if name=='cup_handle' else -1
        if kinds != ('HLHL' if sign==1 else 'LHLH'): return None
        rim1,bottom,rim2,handle=[sign*x for x in prices]
        depth=min(rim1,rim2)-bottom; width=points[2]['index']-first
        if depth <= abs(prices[0])*0.01 or width < 20 or abs(rim1-rim2)>0.2*depth: return None
        if not 0.1*depth <= rim2-handle <= 0.5*depth or last-points[2]['index'] > width*0.6: return None
        location=(points[1]['index']-first)/width
        bowl=[sign*r[4] for r in candles[first:points[2]['index']+1]]
        if not 0.3 <= location <= 0.7 or sum(x <= bottom+0.3*depth for x in bowl) < len(bowl)*0.2: return None
        if sign*candles[first-10][4] > rim1-0.4*depth: return None
        boundary=sign*max(rim1,rim2),0
        return shape(name,points,depth,upper=boundary if sign==1 else None,lower=boundary if sign==-1 else None,
                     direction='bullish' if sign==1 else 'bearish',invalid_lower=prices[3] if sign==1 else None,invalid_upper=prices[3] if sign==-1 else None)
    highs=[p for p in points if p['kind']=='H']; lows=[p for p in points if p['kind']=='L']
    if len(highs)<2 or len(lows)<2: return None
    upper,lower=fit(highs),fit(lows)
    start_width=at(upper,first)-at(lower,first); end_width=at(upper,last)-at(lower,last)
    if start_width <= abs(prices[0])*0.004 or end_width <= start_width*0.15: return None
    if max(abs(p['price']-at(upper,p['index'])) for p in highs)>start_width*0.15: return None
    if max(abs(p['price']-at(lower,p['index'])) for p in lows)>start_width*0.15: return None
    u,l=upper[1]*span/start_width,lower[1]*span/start_width
    ratio=end_width/start_width; converges=0.2 <= ratio <= 0.8
    parallel=abs(u-l)<=0.2 and 0.8<=ratio<=1.2
    pole=(candles[first][4]-candles[first-12][4])/start_width
    matches={
        'ascending_triangle':converges and abs(u)<=0.12 and l>=0.25,
        'descending_triangle':converges and abs(l)<=0.12 and u<=-0.25,
        'symmetrical_triangle':converges and u<=-0.15 and l>=0.15,
        'rising_wedge':converges and u>=0.12 and l>u,
        'falling_wedge':converges and l<=-0.12 and u<l,
        'rectangle':parallel and max(abs(u),abs(l))<=0.12,
        'rising_channel':parallel and min(u,l)>=0.25,
        'falling_channel':parallel and max(u,l)<=-0.25,
        'bull_flag':span<=50 and parallel and -1.2<=min(u,l) and max(u,l)<=0.1 and pole>=1.8,
        'bear_flag':span<=50 and parallel and max(u,l)<=1.2 and min(u,l)>=-0.1 and pole<=-1.8,
        'bull_pennant':span<=50 and converges and u<=-0.15 and l>=0.15 and pole>=1.8,
        'bear_pennant':span<=50 and converges and u<=-0.15 and l>=0.15 and pole<=-1.8,
    }
    if not matches.get(name): return None
    direction='bullish' if name in ('ascending_triangle','falling_wedge','bull_flag','bull_pennant') else 'bearish' if name in ('descending_triangle','rising_wedge','bear_flag','bear_pennant') else 'either'
    return shape(name,points,start_width,upper,lower,direction)
