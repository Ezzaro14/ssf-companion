from pathlib import Path

import pytest
from django.core.management import call_command

from crafting.models import BaseItemType, Mod

FIXTURE = Path(__file__).parent / "fixtures" / "repoe"


@pytest.fixture
def imported(db):
    call_command("import_mod_data", source=str(FIXTURE), league="Test", patch="0.0.0")
                 
def test_suffix_mod_matches_verified_values(imported):
    mod = Mod.objects.get(internal_id="LifeLeechPermyriadSuffix1")
    assert mod.name == "of the Remora"
    assert mod.generation_type == "suffix"
    assert mod.required_level == 50
    weights = [(s.tag.name, s.weight) for s in mod.spawn_weights.all()]
    assert weights == [
        ("ring", 1000),
        ("amulet", 1000),
        ("gloves", 1000),
        ("quiver", 1000),
        ("default", 0),      # allow-list: everything not listed above is blocked
    ]


def test_prefix_mod_matches_verified_values(imported):
    mod = Mod.objects.get(internal_id="IncreasedLife3")
    assert mod.name == "Stalwart"
    assert mod.generation_type == "prefix"
    assert mod.required_level == 18


def test_block_list_ordering_is_preserved(imported):
    """Prudent is blocked on utility and mana flasks, open at 600 elsewhere.

    The zero entries must stay ahead of the positive one - first matching tag
    wins, so reversing this list would invert the mod's meaning entirely.
    """
    mod = Mod.objects.get(internal_id="FlaskIncreasedRecoveryOnLowLife1")
    weights = [(s.tag.name, s.weight) for s in mod.spawn_weights.all()]
    assert weights == [
        ("utility_flask", 0),
        ("mana_flask", 0),
        ("default", 600),
    ]


def test_generation_weights_import(imported):
    mod = Mod.objects.get(internal_id="LifeLeechPermyriadSuffix1")
    assert mod.generation_weights.count() == 7


def test_added_tags_link(imported):
    mod = Mod.objects.get(internal_id="LifeLeechPermyriadSuffix1")
    assert {t.name for t in mod.adds_tags.all()} == {"has_attack_mod"}