import unittest
from summarize import communication, last_lower_read_count, percentile


class AccountingTests(unittest.TestCase):
    def test_cache_is_charged_once_per_generation_not_per_query(self):
        single = communication(1000, 100, 10, 1)
        repeated = communication(1000, 100, 10, 100)
        self.assertEqual(single["cached_download_bytes"], repeated["cached_download_bytes"])
        self.assertTrue(single["pir_estimate_lower"])
        self.assertFalse(repeated["pir_estimate_lower"])

    def test_equality_is_not_reported_as_an_advantage(self):
        self.assertEqual(last_lower_read_count(1000, 100, 10), 89)
        self.assertFalse(communication(1000, 100, 10, 90)["pir_estimate_lower"])
        self.assertEqual(last_lower_read_count(100, 200, 10), 0)
        self.assertEqual(last_lower_read_count(101, 100, 10), 0)
        with self.assertRaises(ValueError):
            last_lower_read_count(100, 10, 0)

    def test_new_snapshots_charge_both_paths_for_each_full_rebuild(self):
        result = communication(1000, 100, 10, 2, 3)
        self.assertEqual(result["pir_min_estimated_bytes"], 360)
        self.assertEqual(result["cached_download_bytes"], 3000)

    def test_nearest_rank_and_invalid_samples(self):
        self.assertEqual(percentile(list(range(1, 33)), 0.95), 31)
        with self.assertRaises(ValueError):
            percentile([], 0.95)
        with self.assertRaises(ValueError):
            communication(1000, 100, 10, 0)


if __name__ == "__main__":
    unittest.main()
