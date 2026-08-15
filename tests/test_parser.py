from pathlib import Path

import pytest

from items.parser import split_sections

FIXTURES = Path(__file__).parent / "fixtures" / "items"  # folder of real copied items


def load(name: str) -> str:
    return (FIXTURES / name).read_text(encoding="utf-8")  # read fixture file as saved


# runs once per .txt file in the fixtures folder
@pytest.mark.parametrize("filename", [p.name for p in FIXTURES.glob("*.txt")])
def test_every_fixture_splits_into_sections(filename):
    sections = split_sections(load(filename))
    assert len(sections) >= 2  # header plus at least one other section
    assert all(section for section in sections)  # no empty sections


def test_first_section_holds_the_header():
    sections = split_sections(load("rare_armour.txt"))
    assert any(line.startswith("Rarity:") for line in sections[0])
