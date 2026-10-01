import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { configFromForm, sameProfile, selectedFeatureCount, chartConfig, chartMatchesModel } from '../ui/modules/ai/settings.js';
const catalog = JSON.parse(fs.readFileSync(new URL('../ai-worker/catalog.json',import.meta.url),'utf8'));

test('paper markers require the chart market, timeframe, inputs and custom rules to match',()=>{
  const rule={left:'rsi',op:'cross_above',right:'number',value:30};
  const model={symbol:'BTCUSDT',interval:'1h',inputs:['rsi','ema'],threshold:0.6,triggers:{buy:[rule],sell:[]}};
  const chart=chartConfig(model);
  assert.ok(chartMatchesModel(chart,model));
  assert.ok(chartMatchesModel({...chart,inputs:['ema','rsi'],triggers:{sell:[],buy:[{value:30,right:'number',op:'cross_above',left:'rsi'}]}},model));
  for(const delta of [{symbol:'ETHUSDT'},{interval:'1m'},{inputs:['rsi']},{triggers:{buy:[{...rule,value:40}],sell:[]}}]) assert.ok(!chartMatchesModel({...chart,...delta},model));
  assert.ok(!sameProfile(model,{...model,threshold:0.7}));
});

test('RSI-only and EMA-only submit precisely the selected inputs',()=>{
  for (const id of ['rsi','ema']) {
    const data = new URLSearchParams({symbol:'btcusdt',interval:'1h',horizon:'4',history_bars:'3000',fee_bps:'10',slippage_bps:'5',threshold:'0.6',take_profit_pct:'1'});
    assert.throws(()=>configFromForm(data),/at least one/);
    data.append('inputs',id);
    const config = configFromForm(data);
    assert.deepEqual(config.inputs,[id]); assert.equal(config.symbol,'BTCUSDT'); assert.equal(config.take_profit_pct,1);
    assert.equal(selectedFeatureCount(catalog,[id]),id === 'rsi' ? 1 : 3);
  }
});

test('combinations survive form submission; profile matching respects input membership and TP',()=>{
  const data = new URLSearchParams({symbol:'BTCUSDT'});
  data.append('inputs','rsi'); data.append('inputs','ema'); data.append('inputs','head_shoulders');
  const config = configFromForm(data);
  assert.deepEqual(config.inputs,['ema','head_shoulders','rsi']);
  const a = {inputs:['rsi','ema'],take_profit_pct:1};
  assert.ok(sameProfile(a,{inputs:['ema','rsi'],take_profit_pct:1}));
  assert.ok(!sameProfile(a,{inputs:['ema'],take_profit_pct:1}));
  assert.ok(!sameProfile(a,{inputs:['ema','rsi'],take_profit_pct:2}));
  assert.ok(!sameProfile(a,null));
});
