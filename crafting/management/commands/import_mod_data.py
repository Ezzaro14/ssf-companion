import json
from pathlib import Path

from django.core.management.base import BaseCommand, CommandError

from crafting.models import DataVersion


class Command(BaseCommand):
    help = "Import base items, mods and spawn weights from a RePoE-format dump"

    def add_arguments(self, parser):
        parser.add_argument("--source", required=True, help="folder holding the json files")
        parser.add_argument("--league", required=True, help='e.g. "Settlers"')
        parser.add_argument("--patch", required=True, help='e.g. "3.25.0"')
        parser.add_argument("--url", default="", help="where the dump came from")
        parser.add_argument("--dry-run", action="store_true", help="report only, write nothing")

    def load(self, source: Path, filename: str) -> dict:
        path = source / filename
        if not path.is_file():
            raise CommandError(f"missing {filename} in {source}")
        with path.open(encoding="utf-8") as f:
            return json.load(f)

    def handle(self, *args, **options):
        source = Path(options["source"])
        if not source.is_dir():
            raise CommandError(f"not a folder: {source}")

        base_items = self.load(source, "base_items.json")
        mods = self.load(source, "mods.json")
        self.stdout.write(f"read {len(base_items)} base items, {len(mods)} mods")

        if options["dry_run"]:
            self.stdout.write(self.style.WARNING("dry run - nothing written"))
            return

        version, created = DataVersion.objects.get_or_create(
            league=options["league"],
            patch=options["patch"],
            defaults={"source": options["url"]},
        )
        verb = "created" if created else "reusing"
        self.stdout.write(f"{verb} data version {version}")
