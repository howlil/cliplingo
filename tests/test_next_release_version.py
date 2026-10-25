import unittest
from tools.next_release_version import next_version


class NextVersionTests(unittest.TestCase):
    def test_first_release(self):
        self.assertEqual(next_version([]), "v0.1.0-beta.1")

    def test_next_existing(self):
        self.assertEqual(next_version(["v0.1.0-beta.1"]), "v0.1.0-beta.2")

    def test_semver_numeric_not_lexicographic(self):
        self.assertEqual(next_version(["v0.1.0-beta.9", "v0.1.0-beta.10"]), "v0.1.0-beta.11")

    def test_ignores_other_versions_and_bad_tags(self):
        self.assertEqual(next_version(["v0.0.8", "v0.1.0", "v0.2.0-beta.30",
                                       "v0.1.0-beta.02", "v0.1.0-beta.-1",
                                       "v0.1.0-beta.4"]), "v0.1.0-beta.5")

    def test_skips_gaps(self):
        self.assertEqual(next_version(["v0.1.0-beta.1", "v0.1.0-beta.7"]),
                         "v0.1.0-beta.8")


if __name__ == "__main__":
    unittest.main()
