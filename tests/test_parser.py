from pathlib import Path

import pytest

from items.parser import (
    extract_mods,
    normalise_mod_text,
    parse_header,
    parse_item,
    parse_sockets,
    split_sections,
)

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


def test_rare_item_has_name_and_base():
    header = parse_header(split_sections(load("rare_armour.txt"))[0])
    assert header.rarity == "Rare"
    assert header.name  # rares always have a rolled name
    assert header.base_type
    assert header.name != header.base_type


def test_magic_item_has_base_only():
    header = parse_header(split_sections(load("magic_flask.txt"))[0])
    assert header.rarity == "Magic"
    assert header.name == ""  # magics never get a separate rolled name
    assert "Flask" in header.base_type


def test_six_socket_item_totals_six():
    sockets = parse_sockets(split_sections(load("rare_armour.txt")))
    assert sum(group.size for group in sockets) == 6  # sockets across all groups


def test_item_without_sockets_returns_empty():
    assert parse_sockets(split_sections(load("magic_flask.txt"))) == []


def test_link_groups_are_separated_by_spaces():
    sockets = parse_sockets(split_sections(load("rare_armour.txt")))
    assert all(group.size >= 1 for group in sockets)  # no empty groups


@pytest.mark.parametrize(
    "text,expected_template,expected_values",
    [
        ("+42 to maximum Life", "+# to maximum Life", [42.0]),
        (
            "Adds 12 to 24 Physical Damage",
            "Adds # to # Physical Damage",
            [12.0, 24.0],
        ),  # ranged mod
        ("40% increased Movement Speed", "#% increased Movement Speed", [40.0]),
        ("Regenerate 1.2 Life per second", "Regenerate # Life per second", [1.2]),  # decimal
        ("Cannot be Frozen", "Cannot be Frozen", []),  # no numbers - must survive untouched
    ],
)
def test_mod_normalisation(text, expected_template, expected_values):
    template, values = normalise_mod_text(text)
    assert template == expected_template
    assert values == expected_values


def test_white_base_has_no_mods():
    assert extract_mods(split_sections(load("white_base.txt"))) == []


def test_unidentified_item_has_no_mods():
    assert extract_mods(split_sections(load("unid_rare.txt"))) == []


def test_crlf_input_parses_identically():
    text = load("rare_armour.txt")
    crlf = text.replace("\n", "\r\n")  # simulate a Windows-style copy
    assert split_sections(text) == split_sections(crlf)  # line ending must not change result


def test_divider_with_trailing_whitespace():
    text = "Rarity: Normal\nIron Ring\n--------   \nItem Level: 1"  # divider with trailing spaces
    assert len(split_sections(text)) == 2


@pytest.mark.parametrize("filename", [p.name for p in FIXTURES.glob("*.txt")])
def test_parse_item_succeeds_on_every_fixture(filename):
    item = parse_item(load(filename))
    assert item.base_type
