import json
from pathlib import Path

from django.core.management.base import BaseCommand, CommandError
from django.db import transaction

from crafting.models import BaseItemType, DataVersion, Tag


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

    @transaction.atomic
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
        n = self.import_base_items(base_items, version)
        self.stdout.write(self.style.SUCCESS(f"imported {n} base items"))

    def import_base_items(self, data: dict, version: DataVersion) -> int:
        count = 0
        for metadata_id, entry in data.items():
            if entry.get("release_state") != "released":
                continue
            if not entry.get("name"):
                continue  # ~400 nameless internal entries

            base = BaseItemType.objects.create(
                data_version=version,
                metadata_id=metadata_id,
                name=entry["name"],
                item_class=entry["item_class"],
                domain=entry["domain"],
            )
            for tag_name in entry.get("tags", []):
                tag, _ = Tag.objects.get_or_create(name=tag_name)
                base.tags.add(tag)
            count += 1
        return count
