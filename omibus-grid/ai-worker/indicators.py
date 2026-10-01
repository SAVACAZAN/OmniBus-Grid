"""Unclipped indicator values for visible chart panels and custom entry rules."""
import math
from statistics import mean, pstdev
from features import smooth, divide
from inputs import BY_ID

OVERLAYS={'sma','ema','bollinger','vwap','donchian','keltner'}


def indicator_series(candles,selected):
    names={name for key in selected for name in BY_ID[key]['series']}
    result={name:[None]*len(candles) for name in names}
    if not candles or not names: return result
    c=[r[4] for r in candles]; h=[r[2] for r in candles]; l=[r[3] for r in candles]; v=[r[5] for r in candles]
    tp=[(a+b+d)/3 for a,b,d in zip(h,l,c)]
    delta=[0.0]+[c[i]-c[i-1] for i in range(1,len(c))]
    gain=smooth([max(x,0) for x in delta],14,True); loss=smooth([max(-x,0) for x in delta],14,True)
    tr=[h[0]-l[0]]+[max(h[i]-l[i],abs(h[i]-c[i-1]),abs(l[i]-c[i-1])) for i in range(1,len(c))]
    atr=smooth(tr,14,True); e12=smooth(c,12); e26=smooth(c,26); e20=smooth(c,20)
    macd=[a-b for a,b in zip(e12,e26)]; signal=smooth(macd,9)
    plus,minus=[0.0],[0.0]
    for i in range(1,len(c)):
        up,down=h[i]-h[i-1],l[i-1]-l[i]
        plus.append(max(up,0) if up>down else 0); minus.append(max(down,0) if down>up else 0)
    pdi=[100*divide(x,a) for x,a in zip(smooth(plus,14,True),atr)]
    mdi=[100*divide(x,a) for x,a in zip(smooth(minus,14,True),atr)]
    adx=smooth([100*divide(abs(a-b),a+b) for a,b in zip(pdi,mdi)],14,True)
    returns=[0]+[math.log(c[i]/c[i-1]) for i in range(1,len(c))]
    for i in range(len(c)):
        if 'volume' in result: result['volume'][i]=v[i]
        if i<100: continue  # Same stabilization window as learner inputs.
        start=i-19; sma=mean(c[start:i+1]); sd=pstdev(c[start:i+1]); volume=sum(v[start:i+1])
        high=max(h[i-13:i+1]); low=min(l[i-13:i+1]); stoch=100*divide(c[i]-low,high-low,0.5)
        m=mean(tp[start:i+1]); dev=mean([abs(x-m) for x in tp[start:i+1]])
        pos=sum(tp[j]*v[j] for j in range(i-13,i+1) if tp[j]>tp[j-1]); neg=sum(tp[j]*v[j] for j in range(i-13,i+1) if tp[j]<tp[j-1])
        ah=max(range(i-24,i+1),key=lambda j:(h[j],j)); al=min(range(i-24,i+1),key=lambda j:(l[j],-j))
        values=dict(sma20=sma,ema12=e12[i],ema26=e26[i],macd=macd[i],macd_signal=signal[i],macd_histogram=macd[i]-signal[i],
            rsi=50 if gain[i]+loss[i]==0 else 100*divide(gain[i],gain[i]+loss[i]),bb_upper=sma+2*sd,bb_mid=sma,bb_lower=sma-2*sd,
            atr=atr[i],stochastic=stoch,williams_r=stoch-100,cci=divide(tp[i]-m,0.015*dev),roc=100*(c[i]/c[i-12]-1),
            momentum=c[i]-c[i-10],adx=adx[i],di_plus=pdi[i],di_minus=mdi[i],mfi=100*divide(pos,pos+neg,0.5),
            obv_flow=divide(sum((1 if delta[j]>0 else -1 if delta[j]<0 else 0)*v[j] for j in range(start,i+1)),volume),
            vwap=divide(sum(tp[j]*v[j] for j in range(start,i+1)),volume,c[i]),
            cmf=divide(sum(divide(2*c[j]-h[j]-l[j],h[j]-l[j])*v[j] for j in range(start,i+1)),volume),
            donchian_high=max(h[start:i+1]),donchian_low=min(l[start:i+1]),keltner_high=e20[i]+2*atr[i],keltner_mid=e20[i],keltner_low=e20[i]-2*atr[i],
            aroon_up=100*(25-(i-ah))/25,aroon_down=100*(25-(i-al))/25,volatility=100*pstdev(returns[start:i+1]),volume=v[i])
        for name in names: result[name][i]=values[name]
    return result


def panels(candles,selected):
    series=indicator_series(candles,selected)
    return [dict(id=key,label=BY_ID[key]['label'],overlay=key in OVERLAYS,
                 series=[dict(name=name,values=series[name]) for name in BY_ID[key]['series']])
            for key in selected if BY_ID[key]['series']]
