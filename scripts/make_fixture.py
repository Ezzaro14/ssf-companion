"""Extract a small subset of the real dump into test fixtures."""

import json
from pathlib import Path

SOURCE = Path("data")
DEST = Path("tests/fixtures/repoe")

# The mods you cross-checked against poedb, plus structural edge cases.
MOD_IDS = [
    "LifeLeechPermyriadSuffix1",
    "FlaskIncreasedRecoveryOnLowLife1",
    "FlaskDispellsChill1",
    "IncreasedLife3",
    "IncreasedLife4",
    "MapMtxRowBoatEnabled",
]

# Base types those mods can roll on, plus one non-flask for contrast.
BASE_IDS = [
    "Metadata/Items/Flasks/FlaskUtility5",
    "Metadata/Items/Armours/BodyArmours/BodyStr17",
]


def subset(filename: str, keys: list[str]) -> dict:
    with (SOURCE / filename).open(encoding="utf-8") as f:
        data = json.load(f)
    missing = [k for k in keys if k not in data]
    if missing:
        raise SystemExit(f"not in {filename}: {missing}")
    return {k: data[k] for k in keys}


DEST.mkdir(parents=True, exist_ok=True)
for filename, keys in [("base_items.json", BASE_IDS), ("mods.json", MOD_IDS)]:
    out = subset(filename, keys)
    with (DEST / filename).open("w", encoding="utf-8") as f:
        json.dump(out, f, indent=2)
    print(f"{filename}: {len(out)} entries")
