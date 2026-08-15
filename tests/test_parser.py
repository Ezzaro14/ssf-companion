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

from items.parser import parse_header


def test_rare_item_has_name_and_base():
    header = parse_header(split_sections(load("rare_armour.txt"))[0])
    assert header.rarity == "Rare"
    assert header.name        # rares always have a rolled name
    assert header.base_type
    assert header.name != header.base_type


def test_magic_item_has_base_only():
    header = parse_header(split_sections(load("magic_flask.txt"))[0])
    assert header.rarity == "Magic"
    assert header.name == ""    # magics never get a separate rolled name
    assert "Flask" in header.base_type
