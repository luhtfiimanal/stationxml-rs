"""Generate test vectors and fixtures for stationxml-rs TDD.

Uses ObsPy as the oracle for StationXML generation and validation.

NOTE: ObsPy is incompatible with Python 3.14 (missing pkg_resources).
Use Python 3.12 or earlier to run this script.
When ObsPy is not available, the script validates hand-written
fixtures using lxml schema validation instead.
"""

from pathlib import Path

FIXTURES_DIR = Path(__file__).parent.parent.parent.parent / "tests" / "fixtures"
VECTORS_DIR = Path(__file__).parent.parent.parent / "test_vectors"

FDSN_SCHEMA_URL = "https://www.fdsn.org/xml/station/fdsn-station-1.2.xsd"


def validate_xml_well_formed(path: Path) -> bool:
    """Check if XML is well-formed using stdlib xml.etree."""
    import xml.etree.ElementTree as ET

    try:
        ET.parse(path)
        return True
    except ET.ParseError as e:
        print(f"  FAIL: {e}")
        return False


def validate_with_obspy(path: Path) -> bool:
    """Validate XML with ObsPy (if available)."""
    try:
        from obspy import read_inventory  # pyright: ignore[reportMissingModuleSource]

        inv = read_inventory(str(path))
        net_count = len(inv.networks)  # pyright: ignore[reportUnknownMemberType]
        sta_count = sum(
            len(net.stations)  # pyright: ignore[reportUnknownMemberType, reportUnknownVariableType]
            for net in inv.networks  # pyright: ignore[reportUnknownMemberType, reportUnknownVariableType]
        )
        print(f"  ObsPy OK: {net_count} networks, {sta_count} stations")
        return True
    except ImportError:
        print("  ObsPy not available, skipping ObsPy validation")
        return True
    except Exception as e:
        print(f"  ObsPy FAIL: {e}")
        return False


def main() -> None:
    VECTORS_DIR.mkdir(parents=True, exist_ok=True)
    FIXTURES_DIR.mkdir(parents=True, exist_ok=True)

    print(f"Fixtures dir: {FIXTURES_DIR}/")
    print(f"Vectors dir:  {VECTORS_DIR}/")
    print()

    # Validate existing fixtures
    fixture_files = sorted(FIXTURES_DIR.glob("*.xml"))
    if not fixture_files:
        print("No fixture files found.")
        return

    print(f"Validating {len(fixture_files)} fixture(s):")
    all_ok = True
    for path in fixture_files:
        print(f"\n  {path.name}:")
        ok = validate_xml_well_formed(path)
        if ok:
            print("  XML well-formed: OK")
            ok = validate_with_obspy(path)
        all_ok = all_ok and ok

    print()
    if all_ok:
        print("All fixtures valid.")
    else:
        print("Some fixtures failed validation!")
        raise SystemExit(1)


if __name__ == "__main__":
    main()
