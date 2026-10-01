import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import unittest
suite = unittest.defaultTestLoader.discover(str(Path(__file__).resolve().parent / 'tests'))
result = unittest.TextTestRunner(verbosity=2).run(suite)
raise SystemExit(not result.wasSuccessful())
