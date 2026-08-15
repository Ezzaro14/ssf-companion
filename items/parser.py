from dataclasses import dataclass, field

DIVIDER = "--------"  # PoE uses 8-dash line to separate item text sections


def split_sections(text: str) -> list[list[str]]:
    """Split raw item text into sections, each a list of non-empty lines."""
    sections: list[list[str]] = []  # finished sections accumulate here
    current: list[str] = []  # collect lines for the section in progress

    for raw_line in text.splitlines():
        line = raw_line.strip()  # drop \r and whitespace
        if line == DIVIDER:
            if current:  # only keep sections with content
                sections.append(current)
            current = []  # collect next section
        elif line:  # ignore blank lines
            current.append(line)

    if current:  # add trailing divider to last section
        sections.append(current)

    return sections


@dataclass
class ParsedItem:
    rarity: str = ""
    item_class: str = ""
    name: str = ""       # rolled name, e.g. "Widowhail" - empty for magic/normal
    base_type: str = ""  # e.g. "Thicket Bow" - always present


def parse_header(section: list[str]) -> ParsedItem:
    item = ParsedItem()
    name_lines: list[str] = []  # leftover lines once labelled fields are pulled out

    for line in section:
        if line.startswith("Item Class:"):
            item.item_class = line.split(":", 1)[1].strip()
        elif line.startswith("Rarity:"):
            item.rarity = line.split(":", 1)[1].strip()
        else:
            name_lines.append(line)  # unlabelled line - one for magic/normal, two for rare/unique

    if len(name_lines) >= 2:
        item.name, item.base_type = name_lines[0], name_lines[1]  # rolled name, then base type
    elif name_lines:
        item.base_type = name_lines[0]  # magic/normal - affixes baked into wording

    return item