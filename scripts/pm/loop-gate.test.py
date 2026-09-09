import sys
from pathlib import Path
import unittest
sys.path.insert(0, str(Path(__file__).parent))
from loop_gate import admission


class GateTests(unittest.TestCase):
    def test_live_legacy_passes(self):
        self.assertEqual(admission(Path('.'), {}, 'base', 'head', reader=lambda _: None)['status'], 'legacy')

    def test_cache_deletion_cannot_downgrade_live_loop(self):
        with self.assertRaisesRegex(ValueError, 'cache differs'):
            admission(Path('.'), {}, 'base', 'head', reader=lambda _: {'loop': 'code'})

    def test_missing_effective_helper_blocks_loop(self):
        binding = {'loop': 'code'}
        with self.assertRaisesRegex(ValueError, 'trusted'):
            admission(Path('.'), {'loop_binding': binding}, 'base', 'head', reader=lambda _: binding)


if __name__ == '__main__': unittest.main()
