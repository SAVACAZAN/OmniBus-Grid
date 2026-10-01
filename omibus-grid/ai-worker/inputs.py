"""One input contract shared by Python, the native backend and the UI."""
import json
from pathlib import Path

CATALOG = json.loads(Path(__file__).with_name('catalog.json').read_text(encoding='utf-8'))
BY_ID = {item['id']: item for item in CATALOG['inputs']}
DEFAULT_INPUTS = CATALOG['default_inputs']


def normalize_inputs(values):
    if not isinstance(values, list) or not values or len(values) > len(BY_ID):
        raise ValueError('Select at least one learning input')
    if any(not isinstance(value, str) or value not in BY_ID for value in values):
        raise ValueError('Unknown learning input')
    if len(set(values)) != len(values):
        raise ValueError('Duplicate learning inputs')
    return sorted(values)


def feature_names(values):
    return sorted({name for value in normalize_inputs(values) for name in BY_ID[value]['features']})
