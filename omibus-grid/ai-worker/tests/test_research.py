import copy
import json
import math
import sqlite3
import tempfile
import unittest
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from engine import Engine, DEFAULTS, bootstrap, example, promotion_decision, independent, TRAIN_PERIOD
from features import feature_rows, INDICATORS, WARMUP
from market import validate, INTERVALS
from model import Model, metrics, net_return

STEP = INTERVALS['1h']
START = 1700000000000 // STEP * STEP


def candles(count=1600):
    result, previous = [], 100.0
    for i in range(count):
        close = 100 + 0.01*i + 2*math.sin(i/15) + math.sin(i/3)
        result.append([START + i*STEP, previous, max(previous, close)+0.4, min(previous, close)-0.4, close, 100+i%17, START+(i+1)*STEP-1])
        previous = close
    return result


class ResearchTests(unittest.TestCase):
    def test_twenty_indicators_and_prefix_invariance(self):
        data = candles(300)
        self.assertEqual(len(INDICATORS), 20)
        prefix = feature_rows(data[:201])[-1]
        complete = feature_rows(data)[200]
        self.assertEqual(prefix, complete)
        self.assertGreater(len(prefix), 30)
        self.assertTrue(all(math.isfinite(x) for row in feature_rows(data)[WARMUP:] for x in row.values()))

    def test_flat_zero_volume_is_finite_and_rsi_neutral(self):
        data = [[START+i*STEP, 100, 100, 100, 100, 0, START+(i+1)*STEP-1] for i in range(150)]
        last = feature_rows(data)[-1]
        self.assertEqual(last['rsi'], 0)
        self.assertEqual(last['bollinger_z'], 0)
        self.assertTrue(all(math.isfinite(x) for x in last.values()))

    def test_candle_quality_gaps_future_and_revisions(self):
        data = candles(150)
        now = data[-1][6]+1
        validate(data, STEP, now)
        with self.assertRaises(ValueError): validate(data, STEP, data[-1][6])
        with self.assertRaises(ValueError): validate(data[:30]+data[31:], STEP, now)
        broken = copy.deepcopy(data); broken[40][2] = 0
        with self.assertRaises(ValueError): validate(broken, STEP, now)
        with tempfile.TemporaryDirectory() as directory:
            engine = Engine(directory, dict(DEFAULTS))
            engine.ingest(data, now)
            changed = copy.deepcopy(data[-1]); changed[5] += 1
            with self.assertRaises(ValueError): engine.ingest([changed], now)
            engine.close()

    def test_costs_and_delayed_entry(self):
        config = dict(DEFAULTS)
        data = candles()
        row = example(data, 100, config)
        self.assertEqual(row['entry_time'], data[102][0])
        self.assertEqual(row['exit_time'], data[105][6])
        self.assertLess(net_return(100, 100, 10, 5), 0)
        self.assertAlmostEqual(net_return(100, 110, 0, 0), 0.1)

    def test_holdout_does_not_choose_or_train_initial_champion(self):
        data = candles()
        state = bootstrap(data, feature_rows(data), dict(DEFAULTS), data[-1][6]+1)
        changed = copy.deepcopy(data)
        for row in changed:
            if row[0] >= state['bootstrap']['holdout_start']:
                for k in range(1, 5): row[k] *= 1.5
        other = bootstrap(changed, feature_rows(changed), dict(DEFAULTS), changed[-1][6]+1)
        self.assertEqual(state['champion'], other['champion'])
        self.assertEqual(state['bootstrap']['selected_features'], other['bootstrap']['selected_features'])

    def test_daily_updates_are_delayed_idempotent_and_restartable(self):
        data = candles(1700)
        with tempfile.TemporaryDirectory() as directory:
            engine = Engine(directory, dict(DEFAULTS))
            first = data[:1600]
            now = first[-1][6]+1000
            engine.ingest(first, now)
            initial = engine.cycle(now)
            latest_signal = initial['signals'][0]
            self.assertLess(latest_signal['issued_at'], latest_signal['entry_time'])
            self.assertNotIn('net', latest_signal)
            self.assertEqual(engine.cycle(now)['training_updates'], initial['training_updates'])
            second = data[1599:1624]
            later = second[-1][6]+1000
            engine.ingest(second, later)
            updated = engine.cycle(later)
            self.assertEqual(updated['training_updates'] - initial['training_updates'], 24)
            saved = json.loads(engine.db.execute('SELECT data FROM state').fetchone()[0])
            self.assertLess(saved['trained_outcome_until'], later)
            self.assertEqual(saved['trained_until'], data[1618][0])
            self.assertEqual(updated['signals'][-1]['p'], latest_signal['p'])
            self.assertIn('net', updated['signals'][-1])
            updates = updated['training_updates']
            engine.close()
            engine = Engine(directory, dict(DEFAULTS))
            resumed = engine.cycle(later)
            self.assertEqual(resumed['training_updates'], updates)
            self.assertEqual(len(resumed['signals']), 2)
            engine.close()

    def test_outage_does_not_invent_past_live_predictions(self):
        data = candles()
        with tempfile.TemporaryDirectory() as directory:
            engine = Engine(directory, dict(DEFAULTS))
            now = data[-1][6]+10*STEP
            engine.ingest(data, now)
            report = engine.cycle(now)
            self.assertEqual(report['signals'], [])
            engine.close()

    def test_slow_training_cannot_issue_a_retroactive_entry(self):
        data = candles()
        with tempfile.TemporaryDirectory() as directory:
            engine = Engine(directory, dict(DEFAULTS))
            now = data[-1][6]+1
            engine.ingest(data, now)
            report = engine.cycle(now, issue_time=lambda: now+2*STEP)
            self.assertEqual(report['signals'], [])
            engine.close()

    def test_promotion_needs_disjoint_forward_evidence_and_both_blocks(self):
        rows = [dict(entry_time=i*10, exit_time=i*10+3, p=0.4, challenger_p=0.9,
                     baseline_p=0.5, short_p=0.1, challenger_short_p=0.01, baseline_short_p=0.5,
                     y=1, short_y=0, net=0.01,
                     long_outcome=dict(net=0.01,exit_time=i*10+3),
                     short_outcome=dict(net=-0.01,exit_time=i*10+3)) for i in range(100)]
        self.assertIsNone(promotion_decision(rows[:99], 0.6))
        self.assertTrue(promotion_decision(rows, 0.6)['promoted'])
        bad = copy.deepcopy(rows)
        for row in bad[50:]:
            row['y'] = 0; row['net'] = -0.01; row['long_outcome']['net'] = -0.01
        self.assertFalse(promotion_decision(bad, 0.6)['promoted'])
        self.assertEqual(len(independent([rows[0], rows[0]])), 1)

    def test_paper_metrics_do_not_stack_overlapping_positions(self):
        rows = [dict(entry_time=i, exit_time=i+3, p=0.9, y=1, net=0.1) for i in range(4)]
        result = metrics(rows)
        self.assertEqual(result['trades'], 1)
        self.assertAlmostEqual(result['return_pct'], 10)
        self.assertIsNone(result['profit_factor'])

    def test_model_checkpoint_roundtrip(self):
        model = Model(['x']); model.learn({'x': 1}, 1)
        restored = Model.load(json.loads(json.dumps(model.dump())))
        self.assertEqual(restored.predict({'x': 2}), model.predict({'x': 2}))


if __name__ == '__main__': unittest.main()
