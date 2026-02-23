"""Generate test vectors and fixtures for stationxml-rs TDD.

Uses ObsPy as the oracle for StationXML generation and validation.
"""

from pathlib import Path

FIXTURES_DIR = Path(__file__).parent.parent.parent.parent / "tests" / "fixtures"
VECTORS_DIR = Path(__file__).parent.parent.parent / "test_vectors"


def main() -> None:
    VECTORS_DIR.mkdir(parents=True, exist_ok=True)
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)
    print(f"Fixtures dir: {FIXTURES_DIR}/")
    print(f"Vectors dir: {VECTORS_DIR}/")
    print("No test vectors to generate yet.")


if __name__ == "__main__":
    main()
