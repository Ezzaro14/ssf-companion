

# Item Formatting - Personal Documentation

Outlines how PoE item information is structured and how `items/parser.py` formats the item.

## Item Copying

With Advanced Mod Descriptions (`AMD` from hereon) turned on, hovering over an item and copying it with Ctrl+C will copy a plain-text block to the clipboard. For current parser testing, distinct item conditions are saved under `tests/fixtures/items/` in .txt documents.

`parse_mod_section` reads that header to set `ParsedMod.kind`, `affix_name`, and `tier`. Items copied without AMD have bare mod lines without additional information and will parse as fallback to `kind="explicit"` without line distinction.

## Structure

PoE item information structure is separated by sections. These are separated by eight dashes (`--------`). `split_sections` strips whitespace and CRLF from every line and drops the dividers, returning a list of sections. 

These sections vary between items and item classes. `find_labelled_value` searches every section for a given `Label:` to distinguish between them.

## Item Header

Each item always has an `Item Class` and `Rarity`, but may contain one or two name lines. Rare and Unique items have a rolled name and a base name of the item itself, parsed as `ParsedItem.name` and `.base_type`.
Magic and Normal items have a single name with modifier names added as suffixes and affixes (e.g. Abecedarian's Basalt Flask of Bloodshed, where Basalt Flask is the base item). Only `.base_type` is set. May look into removing appended mod names in the future, as the mods themselves are still found under `parsedMod.affix_name`.

## Sockets

Linked sockets are connected with dashes. Others separated by space (e.g `Sockets: R-G-B-W W-B`). `parse_sockets` returns `list[SocketGroup]` and `SocketGroup.size` returns socket count in the group. Items without sockets return `[]`

## Flags

Certain special modifiers are captured only as bare lines (e.g. `corrupted`, `Mirrored`, `Unidentified`, etc.). `parse_flags` matches these mods against a **harcoded whitelist** (`items/parser.py::FLAGS`). Will look into a solution in the future for future-league proofing.

## Tested So Far

- Magic Utility Flask
- Rare Body Armour
- 6 Socket Body Armour
- Benchcrafted Rare
- Unidentified Rare
- Unique
- Corrupted Rare
- Corrupted Unique with rerolled implicit
- Anointed Amulet
- Item with quality 
- White item base

## Things To Remember

- Lines are copied with `\r\n` appended. `.strip()` is necessary to avoid breaking divider matching in `split_sections`
- Be aware of Benchcrafted Mod Headers. `{ Prefix Modifier "Flaring"}` vs `{ Master Crafted Prefix Modifier "Upgraded"}`. Capturing only a single `\w+` for `kind` silently fails to match the crafted case and misparses it. 
- Numbers extraction must separate `+` and `-` signs from the digits. `"# to maximum Life"` from `[+-]?\d+` vs `"+# to maximum Life"`