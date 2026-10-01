"""Custom indicator conditions filter learned entries; they never place orders."""
import math
from inputs import BY_ID

OPS=('above','below','cross_above','cross_below')


def validate_triggers(value,inputs):
    if not isinstance(value,dict) or set(value)!={'buy','sell'}: raise ValueError('Invalid trigger groups')
    allowed={name for key in inputs for name in BY_ID[key]['series']}
    result={}
    for side in ('buy','sell'):
        rules=value[side]
        if not isinstance(rules,list) or len(rules)>4: raise ValueError('Use at most four conditions per side')
        normalized=[]
        for rule in rules:
            if not isinstance(rule,dict) or set(rule)!={'left','op','right','value'}: raise ValueError('Invalid trigger condition')
            if rule['left'] not in allowed or rule['op'] not in OPS or rule['right'] not in allowed|{'number'}: raise ValueError('Trigger indicators must be selected as learning inputs')
            number=rule['value']
            if type(number) not in (int,float) or not math.isfinite(number) or abs(number)>1e12: raise ValueError('Invalid trigger level')
            normalized.append({**rule,'value':float(number) if rule['right']=='number' else 0.0})
        result[side]=normalized
    return result


def condition(rule,series,index):
    left=series[rule['left']]; right=series.get(rule['right'])
    a=left[index]; b=rule['value'] if right is None else right[index]
    if a is None or b is None: return False
    if rule['op']=='above': return a>b
    if rule['op']=='below': return a<b
    if index==0 or left[index-1] is None or (right is not None and right[index-1] is None): return False
    pa=left[index-1]; pb=rule['value'] if right is None else right[index-1]
    return pa<=pb and a>b if rule['op']=='cross_above' else pa>=pb and a<b


def eligibility(config,series,index):
    return {side:all(condition(rule,series,index) for rule in config['triggers'][side]) for side in ('buy','sell')}
