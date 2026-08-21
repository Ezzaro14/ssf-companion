import json
from collections import Counter
from pathlib import Path

from django.core.management.base import BaseCommand, CommandError
from django.db import transaction

from crafting.models import (
    BaseItemType,
    DataVersion,
    GenerationWeight,
    Mod,
    ModGroup,
    SpawnWeight,
    Tag,
)


class Command(BaseCommand):
    help = "Import base items, mods and spawn weights from a RePoE-format dump"

    def add_arguments(self, parser):
        parser.add_argument("--source", required=True, help="folder holding the json files")
        parser.add_argument("--league", required=True, help='e.g. "Settlers"')
        parser.add_argument("--patch", required=True, help='e.g. "3.25.0"')
        parser.add_argument("--url", default="", help="where the dump came from")
        parser.add_argument("--dry-run", action="store_true", help="report only, write nothing")
        parser.add_argument(
            "--force",
            action="store_true",
            help="wipe and reimport a data version that already exists",
        )

    def load(self, source: Path, filename: str) -> dict:
        path = source / filename
        if not path.is_file():
            raise CommandError(f"missing {filename} in {source}")
        with path.open(encoding="utf-8") as f:
            return json.load(f)

    # tags

    def collect_tag_names(self, base_items: dict, mods: dict) -> set[str]:
        """Every tag name referenced anywhere in the dump."""
        names: set[str] = set()

        for entry in base_items.values():
            names |= set(entry.get("tags") or [])

        for entry in mods.values():
            names |= {sw["tag"] for sw in entry.get("spawn_weights") or []}
            names |= {gw["tag"] for gw in entry.get("generation_weights") or []}
            names |= set(entry.get("implicit_tags") or [])
            names |= set(entry.get("adds_tags") or [])

        return names

    def build_tag_cache(self, base_items: dict, mods: dict) -> dict[str, Tag]:
        # Create every tag up front, then return a name -> Tag lookup
        # Avoids a get_or_create round trip per tag per row; with ~40k mods

        names = self.collect_tag_names(base_items, mods)
        Tag.objects.bulk_create([Tag(name=n) for n in names], ignore_conflicts=True)
        return {t.name: t for t in Tag.objects.all()}

    # ------------------------------------------------------------------

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

        if created:
            self.stdout.write(f"created data version {version}")
        else:
            if not options["force"]:
                raise CommandError(
                    f"{version} already imported - pass --force to wipe and reimport"
                )
            self.stdout.write(self.style.WARNING(f"clearing existing rows for {version}"))
            # SpawnWeight, GenerationWeight and the m2m rows cascade with their parents
            Mod.objects.filter(data_version=version).delete()
            BaseItemType.objects.filter(data_version=version).delete()

        self.tags = self.build_tag_cache(base_items, mods)
        self.stdout.write(f"cached {len(self.tags)} tags")

        n = self.import_base_items(base_items, version)
        self.stdout.write(self.style.SUCCESS(f"imported {n} base items"))

        m = self.import_mods(mods, version)
        self.stdout.write(self.style.SUCCESS(f"imported {m} mods"))

        a = self.import_added_tags(mods, version)
        self.stdout.write(self.style.SUCCESS(f"linked added tags on {a} mods"))

    def import_base_items(self, data: dict, version: DataVersion) -> int:
        # Import released, named base items
        count = 0
        for metadata_id, entry in data.items():
            if entry.get("release_state") != "released":
                continue
            if not entry.get("name"):
                continue

            base = BaseItemType.objects.create(
                data_version=version,
                metadata_id=metadata_id,
                name=entry["name"],
                item_class=entry["item_class"],
                domain=entry["domain"],
            )
            tags = [self.tags[name] for name in entry.get("tags") or []]
            if tags:
                base.tags.add(*tags)
            count += 1
        return count

    def import_mods(self, data: dict, version: DataVersion) -> int:
        seen = Counter(v["generation_type"] for v in data.values())
        self.stdout.write(f"generation types: {dict(seen.most_common())}")
        count = 0

        for internal_id, entry in data.items():
            mod = Mod.objects.create(
                data_version=version,
                internal_id=internal_id,
                name=entry.get("name", ""),
                generation_type=entry["generation_type"],
                domain=entry["domain"],
                required_level=entry.get("required_level", 1),
            )

            # "groups" list or single "group"; no group means it blocks only itself
            group_names = entry.get("groups") or ([entry["group"]] if entry.get("group") else [])
            if not group_names:
                group_names = [internal_id]
            for group_name in group_names:
                group, _ = ModGroup.objects.get_or_create(name=group_name)
                mod.groups.add(group)

            # mod's own tags - attribute, attack, caster, and so on
            mod_tags = [self.tags[name] for name in entry.get("implicit_tags") or []]
            if mod_tags:
                mod.tags.add(*mod_tags)

            SpawnWeight.objects.bulk_create(
                [
                    SpawnWeight(
                        mod=mod,
                        tag=self.tags[sw["tag"]],
                        weight=sw["weight"],
                        order=position,
                    )
                    for position, sw in enumerate(entry.get("spawn_weights") or [])
                ]
            )

            GenerationWeight.objects.bulk_create(
                [
                    GenerationWeight(
                        mod=mod,
                        tag=self.tags[gw["tag"]],
                        value=gw["weight"],
                        order=position,
                    )
                    for position, gw in enumerate(entry.get("generation_weights") or [])
                ]
            )

            count += 1

        return count

    def import_added_tags(self, data: dict, version: DataVersion) -> int:
        """Link mods to the tags they add to an item.

        Must run after import_mods: a mod can add a tag that other mods in the
        same dump depend on, so every Mod row has to exist first.
        """
        mods_by_id = {m.internal_id: m for m in Mod.objects.filter(data_version=version)}
        count = 0

        for internal_id, entry in data.items():
            added = entry.get("adds_tags") or []
            if not added:
                continue
            mod = mods_by_id[internal_id]
            mod.adds_tags.add(*[self.tags[name] for name in added])
            count += 1

        return count
