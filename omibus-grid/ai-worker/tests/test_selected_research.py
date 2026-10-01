import copy
import json
import math
import tempfile
import unittest
from pathlib import Path

from test_research import candles, STEP, START
from engine import Engine, DEFAULTS, bootstrap, outcome, profile_id, validate_config
from features import feature_rows
from inputs import BY_ID, feature_names
from patterns import scan_patterns
from model import direction, directional_return, target_price, directional_metrics


def formation(head=True, inverse=False, breakout=True):
    anchors = [(0,85),(90,90),(105,96),(120,120)]
    anchors += [(130,110),(140,135),(150,110),(160,121),(174,104 if breakout else 112),(200,103 if breakout else 113)] if head else [(135,105),(150,119.5),(169,99 if breakout else 110),(200,98 if breakout else 111)]
    result = []
    for (start,a),(end,b) in zip(anchors,anchors[1:]):
        for i in range(start,end):
            price = a+(b-a)*(i-start)/(end-start)
            if inverse: price = 300-price
            result.append([START+i*STEP,price,price+0.1,price-0.1,price,100,START+(i+1)*STEP-1])
    return result


class SelectionAndPatternTests(unittest.TestCase):
    def test_every_input_has_exactly_its_declared_features(self):
        data = candles(160)
        for name,item in BY_ID.items():
            with self.subTest(name=name):
                rows = feature_rows(data,[name])
                self.assertEqual(set(rows[-1]),set(item['features']))
                self.assertTrue(all(math.isfinite(v) for v in rows[-1].values()))
        self.assertEqual(set(feature_rows(data,['rsi'])[-1]),{'rsi'})
        self.assertEqual(set(feature_rows(data,['ema'])[-1]),{'ema_spread','ema_fast_distance','ema_slow_distance'})

    def test_custom_choice_is_not_replaced_and_profiles_are_canonical(self):
        config = {**DEFAULTS,'inputs':['rsi']}
        data = candles()
        state = bootstrap(data,feature_rows(data,config['inputs']),config,data[-1][6]+1)
        self.assertEqual(state['champion']['long']['names'],['rsi'])
        self.assertEqual(state['champion']['short']['names'],['rsi'])
        self.assertEqual(state['learner']['long']['names'],['rsi'])
        self.assertEqual(profile_id({**config,'inputs':['rsi','ema']}),profile_id({**config,'inputs':['ema','rsi']}))
        self.assertNotEqual(profile_id(config),profile_id({**config,'inputs':['ema']}))
        self.assertNotEqual(profile_id(config),profile_id({**config,'take_profit_pct':2}))
        for values in ([],['unknown'],['rsi','rsi'],'rsi',[12]):
            with self.assertRaises(ValueError): validate_config({**config,'inputs':values})

    def test_all_four_formations_require_neckline_confirmation(self):
        for head,inverse,name in [(True,False,'head_shoulders'),(True,True,'inverse_head_shoulders'),(False,False,'double_top'),(False,True,'double_bottom')]:
            with self.subTest(name=name):
                data = formation(head,inverse)
                rows,events = scan_patterns(data,[name])
                self.assertEqual(len(events),1)
                event = events[0]
                self.assertGreaterEqual(event['confirmed_index'],163 if head else 153)
                self.assertEqual(event['direction'],'bullish' if inverse else 'bearish')
                self.assertEqual(rows[event['confirmed_index']]['pattern_'+name],1)
                self.assertEqual(scan_patterns(formation(head,inverse,False),[name])[1],[])

    def test_formations_and_features_never_repaint_past_rows(self):
        data = formation()
        complete,events = scan_patterns(data)
        for end in [140,162,170,175,180,195]:
            prefix,prefix_events = scan_patterns(data[:end])
            self.assertEqual(prefix,complete[:end])
            self.assertEqual(prefix_events,[e for e in events if e['confirmed_index'] < end])
            self.assertEqual(feature_rows(data[:end],['rsi','head_shoulders'])[-1],feature_rows(data,['rsi','head_shoulders'])[end-1])

    def test_wrong_shoulder_geometry_and_flat_charts_are_rejected(self):
        data = formation()
        # Lower the head into an ordinary shoulder: no higher middle peak remains.
        for i in range(131,150):
            for key in range(1,5): data[i][key] -= 16*max(0,1-abs(i-140)/10)
        self.assertEqual(scan_patterns(data,['head_shoulders'])[1],[])
        flat = [[START+i*STEP,100,100,100,100,0,START+(i+1)*STEP-1] for i in range(200)]
        self.assertEqual(scan_patterns(flat)[1],[])


class DirectionAndPositionTests(unittest.TestCase):
    def test_buy_sell_wait_use_two_learned_scores(self):
        self.assertEqual(direction(0.8,0.2,0.6),'BUY')
        self.assertEqual(direction(0.2,0.8,0.6),'SELL')
        self.assertEqual(direction(0.8,0.9,0.6),'SELL')
        self.assertEqual(direction(0.5,0.4,0.6),'WAIT')
        self.assertEqual(direction(0.8,0.8,0.6),'WAIT')

    def test_long_and_short_tp_close_at_net_target_including_costs(self):
        for side in ('LONG','SHORT'):
            target = target_price(100,1,10,5,side)
            self.assertAlmostEqual(directional_return(100,target,10,5,side),0.01)
            data = [[START,100,105,95,100,100,START+STEP-1]]
            result = outcome(data,0,DEFAULTS,side,partial=True)
            self.assertEqual(result['reason'],'TP')
            self.assertAlmostEqual(result['net'],0.01)
        self.assertGreater(directional_return(100,90,10,5,'SHORT'),0)
        self.assertLess(directional_return(100,110,10,5,'SHORT'),0)

    def test_time_exit_is_not_mislabelled_tp_and_waits_for_closed_history(self):
        data = [[START+i*STEP,100,100.1,99.5,99.8,100,START+(i+1)*STEP-1] for i in range(4)]
        self.assertIsNone(outcome(data[:3],0,DEFAULTS,'LONG',partial=True))
        result = outcome(data,0,DEFAULTS,'LONG')
        self.assertEqual(result['reason'],'TIME EXIT')
        self.assertLess(result['net'],0)

    def test_short_metrics_are_nonoverlapping_and_use_short_outcome(self):
        rows = [dict(entry_time=i,exit_time=i+3,p=0.1,short_p=0.9,y=0,short_y=1,
                     long_outcome=dict(net=-0.1,exit_time=i+3),short_outcome=dict(net=0.1,exit_time=i+3)) for i in range(4)]
        result = directional_metrics(rows)
        self.assertEqual(result['trades'],1)
        self.assertAlmostEqual(result['return_pct'],10)

    def test_live_paper_buy_and_sell_recover_and_close_once_at_tp(self):
        for side in ('LONG','SHORT'):
            with self.subTest(side=side), tempfile.TemporaryDirectory() as directory:
                config = {**DEFAULTS,'inputs':['rsi']}
                data = candles(1610)
                engine = Engine(directory,config)
                first = data[:1600]
                now = first[-1][6]+1
                engine.ingest(first,now)
                state = bootstrap(first,feature_rows(first,config['inputs']),engine.config,now)
                for key in ('champion','challenger'):
                    for direction_name in ('long','short'):
                        state[key][direction_name]['weights'] = [0.0]
                        state[key][direction_name]['bias'] = 5 if direction_name.upper() == side else -5
                with engine.db:
                    engine.db.execute('INSERT INTO state VALUES (1,?)',(json.dumps(state),))
                report = engine.cycle(now)
                self.assertEqual(report['signals'][0]['signal'],'BUY' if side == 'LONG' else 'SELL')
                self.assertEqual(report['position']['status'],'PENDING')
                self.assertGreater(report['position']['entry_time'],now)
                # A stale provider response must not invent an outcome or crash,
                # even if exchange time says that the holding period has elapsed.
                late = engine.cycle(now+8*STEP)
                self.assertNotIn('net',late['signals'][0])
                self.assertEqual(late['position']['status'],'PENDING')
                engine.cycle(now)
                self.assertEqual(engine.db.execute('SELECT COUNT(*) FROM trades').fetchone()[0],1)
                engine.close()
                engine = Engine(directory,config)
                engine.ingest(data[1600:1601],data[1600][6]+1)
                waiting = engine.cycle(data[1600][6]+1)
                self.assertEqual(waiting['signals'][0]['signal'],'HOLD '+side)
                self.assertEqual(waiting['position']['status'],'PENDING')
                bar = copy.deepcopy(data[1601]); bar[2] = max(bar[2],bar[1]*1.05); bar[3] = min(bar[3],bar[1]*0.95)
                engine.ingest([bar],bar[6]+1)
                closed = engine.cycle(bar[6]+1)
                finished = [t for t in closed['trades'] if t['status'] == 'CLOSED']
                self.assertEqual(len(finished),1)
                self.assertEqual(finished[0]['reason'],'TP')
                self.assertAlmostEqual(finished[0]['net'],0.01)
                self.assertEqual(closed['forward']['trades'],1)
                self.assertEqual(engine.cycle(bar[6]+1)['forward']['trades'],1)
                self.assertEqual(closed['feature_names'],['rsi'])
                engine.close()

    def test_v1_research_is_not_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            legacy = Path(directory)/'original-profile'
            legacy.mkdir(); (legacy/'research.sqlite').write_bytes(b'old research')
            engine = Engine(directory,DEFAULTS)
            self.assertNotEqual(engine.directory,legacy)
            engine.close()
            self.assertEqual((legacy/'research.sqlite').read_bytes(),b'old research')


if __name__ == '__main__': unittest.main()
