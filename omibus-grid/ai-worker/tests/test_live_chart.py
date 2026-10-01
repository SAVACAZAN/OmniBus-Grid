import copy
import math
import unittest
from unittest.mock import patch
from test_research import candles, START, STEP
from test_selected_research import formation
from inputs import BY_ID
from patterns import scan_patterns
from indicators import indicator_series, panels
from triggers import validate_triggers, eligibility
from chart_worker import chart_snapshot
from engine import DEFAULTS, bootstrap, allowed_rows, profile_id
from features import feature_rows
from model import direction, directional_metrics
from market import validate, INTERVALS, alignment_offset, fetch


def fixture(name):
    if name in ('head_shoulders','inverse_head_shoulders','double_top','double_bottom'):
        return formation('shoulders' in name,name in ('inverse_head_shoulders','double_bottom'))
    mirror=name in ('triple_bottom','descending_triangle','falling_wedge','bear_flag','bear_pennant','falling_channel','inverse_cup_handle')
    equivalent={'triple_bottom':'triple_top','descending_triangle':'ascending_triangle','falling_wedge':'rising_wedge','bear_flag':'bull_flag','bear_pennant':'bull_pennant','falling_channel':'rising_channel','inverse_cup_handle':'cup_handle'}.get(name,name)
    shapes={
        'triple_top':([120,105,120,105,120],110,102),
        'ascending_triangle':([120,100,120,108,120],117,123),
        'symmetrical_triangle':([120,100,116,104,112],109,115),
        'rising_wedge':([110,100,116,108,122],118,110),
        'rectangle':([120,100,120,100,120],115,125),
        'rising_channel':([110,94,122,106,134],123,112),
        'bull_flag':([130,116,126,112,122],118,125),
        'bull_pennant':([140,120,136,124,132],129,135),
    }
    if equivalent=='cup_handle':
        anchors=[(0,70),(108,80),(120,120),(140,100),(160,119),(170,112),(178,117),(180,118),(181,123),(200,124)]
    else:
        values,inside,breakout=shapes[equivalent]
        anchors=[(0,70),(108,80)]+list(zip(range(120,161,10),values))+[(166,inside),(170,inside),(171,breakout),(200,breakout)]
    data=[]
    for (a,p),(b,q) in zip(anchors,anchors[1:]):
        for i in range(a,b):
            price=p+(q-p)*(i-a)/(b-a)
            if mirror: price=300-price
            data.append([START+i*STEP,price,price+0.1,price-0.1,price,100,START+(i+1)*STEP-1])
    return data


class LiveChartTests(unittest.TestCase):
    def test_twenty_detectors_confirm_positive_fixtures_and_preserve_prefixes(self):
        names=[key for key,item in BY_ID.items() if item['kind']=='pattern']
        self.assertEqual(len(names),20)
        for name in names:
            with self.subTest(pattern=name):
                data=fixture(name)
                rows,events=scan_patterns(data,[name])
                self.assertTrue(events, name+' must actually detect its positive fixture')
                for event in events:
                    if event['direction']=='bullish': self.assertGreater(event['bull_trigger'],event['neckline'])
                    else: self.assertLess(event['bear_trigger'],event['neckline'])
                first=events[0]['confirmed_index']
                prefix,past,forming=scan_patterns(data[:first],[name],True)
                self.assertEqual(past,[])
                self.assertTrue(forming, name+' should be shown forming before its break')
                self.assertEqual(rows[:first],prefix)
                self.assertTrue(all(r['confirmed_at']>r['points'][-1]['time']+2*STEP for r in events))

    def test_no_selected_patterns_produces_no_formation_overlay(self):
        self.assertEqual(scan_patterns(fixture('head_shoulders'),['rsi'],True)[1:],([],[]))

    def test_every_indicator_has_visible_unclipped_finite_series(self):
        data=candles(180)
        for key,item in BY_ID.items():
            if item['kind']=='pattern':continue
            with self.subTest(indicator=key):
                rendered=panels(data,[key])
                self.assertEqual(len(rendered),1)
                self.assertEqual({s['name'] for s in rendered[0]['series']},set(item['series']))
                for s in rendered[0]['series']:
                    self.assertEqual(len(s['values']),len(data));self.assertTrue(math.isfinite(s['values'][-1]))
        series=indicator_series(data,['ema','rsi'])
        self.assertGreater(series['ema12'][-1],90)
        self.assertTrue(0<=series['rsi'][-1]<=100)
        self.assertEqual(indicator_series(data[:150],['rsi'])['rsi'][-1],series['rsi'][149])

    def test_provisional_bar_cannot_confirm_pattern_or_fire_custom_rule(self):
        data=fixture('head_shoulders')
        event=scan_patterns(data,['head_shoulders'])[1][0]
        i=event['confirmed_index']
        config=dict(symbol='BTCUSDT',interval='1h',inputs=['head_shoulders','rsi'],triggers={'buy':[],'sell':[]})
        report=chart_snapshot(config,data[:i+1],data[i][0]+STEP//2)
        self.assertTrue(report['provisional']);self.assertEqual(report['confirmed'],[])
        self.assertTrue(report['forming'])
        complete=chart_snapshot(config,data[:i+1],data[i][6]+1)
        self.assertFalse(complete['provisional']);self.assertTrue(complete['confirmed'])

    def test_weekly_alignment_is_monday_and_supported_intervals_are_consistent(self):
        for step in INTERVALS.values():
            t=alignment_offset(step)+20*step
            validate([[t,100,101,99,100,1,t+step-1]],step,t+step)
        with patch('market.request',side_effect=[{'serverTime':1790629594231},[]]) as request:
            fetch('BTCUSDT','1w',3000)
            self.assertEqual(request.call_args.args[1]['startTime'],0)

    def test_custom_rule_waits_for_the_current_candle_to_close(self):
        data=candles(160)
        values=indicator_series(data,['rsi'])['rsi']
        self.assertNotEqual(values[-1],values[-2])
        rule=dict(left='rsi',op='above' if values[-1]>values[-2] else 'below',right='number',value=(values[-1]+values[-2])/2)
        config=dict(symbol='BTCUSDT',interval='1h',inputs=['rsi'],triggers={'buy':[rule],'sell':[]})
        provisional=chart_snapshot(config,data,data[-1][0]+STEP//2)
        self.assertFalse(provisional['rule_status']['buy'])
        self.assertFalse(any(m['time']==data[-1][0] for m in provisional['custom_triggers']))
        complete=chart_snapshot(config,data,data[-1][6]+1)
        self.assertTrue(complete['rule_status']['buy'])
        self.assertTrue(any(m['time']==data[-1][0] for m in complete['custom_triggers']))

    def test_chart_has_no_research_state_side_effects(self):
        data=candles(160)
        config=dict(symbol='BTCUSDT',interval='1h',inputs=['rsi'],triggers={'buy':[],'sell':[]})
        report=chart_snapshot(config,data,data[-1][6]+1)
        self.assertEqual([p['id'] for p in report['panels']],['rsi'])
        self.assertEqual(report['config']['inputs'],['rsi'])
        with self.assertRaisesRegex(ValueError,'at least 101'):chart_snapshot(config,data[:50],data[-1][6]+1)
        with self.assertRaises(ValueError):chart_snapshot({**config,'symbol':'BTCUSDT;cmd'},data,data[-1][6]+1)


class CustomTriggerTests(unittest.TestCase):
    def test_crossings_and_indicator_comparisons(self):
        series={'rsi':[25,30,31,32,29],'ema12':[9,10,12,11,9],'ema26':[10,10,10,10,10]}
        config={'triggers':{'buy':[dict(left='rsi',op='cross_above',right='number',value=30)],'sell':[]}}
        self.assertFalse(eligibility(config,series,1)['buy'])
        self.assertTrue(eligibility(config,series,2)['buy'])
        self.assertFalse(eligibility(config,series,3)['buy'])
        config['triggers']['sell']=[dict(left='ema12',op='cross_below',right='ema26',value=0)]
        self.assertTrue(eligibility(config,series,4)['sell'])
        config['triggers']['buy'].append(dict(left='ema12',op='below',right='ema26',value=0))
        self.assertFalse(eligibility(config,series,2)['buy'])

    def test_invalid_or_unselected_indicator_conditions_are_rejected(self):
        rule=dict(left='rsi',op='above',right='number',value=30)
        self.assertEqual(validate_triggers({'buy':[rule],'sell':[]},['rsi'])['buy'][0]['value'],30)
        with self.assertRaises(ValueError): validate_triggers({'buy':[rule],'sell':[]},['ema'])
        for config in [{'buy':[{**rule,'op':'execute'}],'sell':[]},{'buy':[{**rule,'value':float('nan')}],'sell':[]}]:
            with self.assertRaises(ValueError): validate_triggers(config,['rsi'])

    def test_filters_apply_to_historical_simulation_and_profile_identity(self):
        blocked={**DEFAULTS,'inputs':['rsi'],'triggers':{'buy':[dict(left='rsi',op='above',right='number',value=101)],'sell':[dict(left='rsi',op='below',right='number',value=-1)]}}
        data=candles(1600)
        allowed=allowed_rows(data,blocked)
        self.assertFalse(any(row['buy'] or row['sell'] for row in allowed))
        self.assertEqual(direction(.9,.8,.6,False,False),'WAIT')
        state=bootstrap(data,feature_rows(data,['rsi']),blocked,data[-1][6]+1)
        self.assertEqual(state['bootstrap']['holdout']['trades'],0)
        self.assertNotEqual(profile_id(blocked),profile_id({**blocked,'triggers':{'buy':[],'sell':[]}}))


if __name__=='__main__':unittest.main()
