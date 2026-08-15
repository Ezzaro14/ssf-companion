DIVIDER = "--------"


def split_sections(text: str) -> list[list[str]]:
    sections: list[list[str]] = []
    current: list[str] = []

    for raw_line in text.splitlines():
        line = raw_line.strip()
        if line == DIVIDER:
            if current:
                sections.append(current)
            current = []
        elif line:
            current.append(line)

    if current:
        sections.append(current)

    return sections
