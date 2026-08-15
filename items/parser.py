import re
from dataclasses import dataclass, field

DIVIDER = "--------"  # PoE uses 8-dash line to separate item text sections

MOD_HEADER = re.compile(  # matches: { Prefix Modifier "Flaring" (Tier: 3) }
    r"^\{\s*(?P<kind>\w+)\s+Modifier"
    r'(?:\s+"(?P<name>[^"]+)")?'
    r"(?:\s+\(Tier:\s*(?P<tier>\d+)\))?"
)

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
class SocketGroup:
    colours: list[str]  # e.g. ["B", "G", "R"] for one linked group

    @property
    def size(self) -> int:
        return len(self.colours)


@dataclass
class ParsedItem:
    rarity: str = ""
    item_class: str = ""
    name: str = ""       # rolled name, e.g. "Widowhail" - empty for magic/normal
    base_type: str = ""  # e.g. "Thicket Bow" - always present
    item_level: int | None = None
    quality: int = 0
    sockets: list[SocketGroup] = field(default_factory=list)

def find_labelled_value(sections: list[list[str]], label: str) -> str | None:
    prefix = f"{label}:"
    for section in sections:      # item level can live in any section
        for line in section:
            if line.startswith(prefix):
                return line.split(":", 1)[1].strip()
    return None  # label not found


def parse_item_level(sections: list[list[str]]) -> int | None:
    value = find_labelled_value(sections, "Item Level")
    return int(value) if value else None

def parse_quality(sections: list[list[str]]) -> int:
    value = find_labelled_value(sections, "Quality")
    if not value:
        return 0  # no Quality line - treat as 0%
    digits = "".join(c for c in value.split("%")[0] if c.isdigit())  # "+20% (augmented)" -> "20"
    return int(digits) if digits else 0

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

def parse_sockets(sections: list[list[str]]) -> list[SocketGroup]:
    value = find_labelled_value(sections, "Sockets")
    if not value:
        return []  # no Sockets line (flasks, jewels)
    return [
        SocketGroup(colours=group.split("-"))  # spaces split groups, dashes split sockets
        for group in value.split()
    ]

@dataclass
class ParsedMod:
    text: str
    kind: str = "explicit"      # implicit / prefix / suffix / enchant / crafted
    affix_name: str = ""        # e.g. "Flaring" - empty if unknown
    tier: int | None = None     # e.g. 3 - None if unknown


def parse_mod_section(section: list[str]) -> list[ParsedMod]:
    mods: list[ParsedMod] = []
    pending: dict | None = None

    for line in section:
        match = MOD_HEADER.match(line)
        if match:
            pending = match.groupdict()  # header line
            continue
        if pending:
            mods.append(ParsedMod(  # mod text
                text=line,
                kind=pending["kind"].lower(),
                affix_name=pending["name"] or "",
                tier=int(pending["tier"]) if pending["tier"] else None,
            ))
            pending = None  # consumed, reset for next mod
        else:
            mods.append(ParsedMod(text=line))  # no header matched

    return mods