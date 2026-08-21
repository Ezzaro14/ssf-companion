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

    def tag_cache(self, names: set[str]) -> dict[str, Tag]:
        existing = {t.name: t for t in Tag.objects.filter(name__in=names)}
        missing = names - existing.keys()
        if missing:
            Tag.objects.bulk_create([Tag(name=n) for n in missing], batch_size=1000)
            existing.update({t.name: t for t in Tag.objects.filter(name__in=missing)})
        return existing

    def group_cache(self, names: set[str]) -> dict[str, int]:
        # Same pattern as tag_cache, but only ids
        existing = dict(ModGroup.objects.filter(name__in=names).values_list("name", "id"))
        missing = names - existing.keys()
        if missing:
            ModGroup.objects.bulk_create([ModGroup(name=n) for n in missing], batch_size=1000)
            existing.update(
                dict(ModGroup.objects.filter(name__in=missing).values_list("name", "id"))
            )
        return existing

    def bulk_link(self, through, source_field: str, target_field: str, pairs) -> None:
        through.objects.bulk_create(
            [through(**{source_field: s, target_field: t}) for s, t in pairs],
            batch_size=5000,
            ignore_conflicts=True,
        )


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

        version = self.resolve_data_version(options)

        self.tags = self.tag_cache(self.collect_tag_names(base_items, mods))
        self.stdout.write(f"cached {len(self.tags)} tags")

        n = self.import_base_items(base_items, version)
        self.stdout.write(self.style.SUCCESS(f"imported {n} base items"))
 
        m = self.import_mods(mods, version)
        self.stdout.write(self.style.SUCCESS(f"imported {m} mods"))
 
        a = self.import_added_tags(mods, version)
        self.stdout.write(self.style.SUCCESS(f"linked added tags on {a} mods"))

    def resolve_data_version(self, options) -> DataVersion:
        version, created = DataVersion.objects.get_or_create(
            league=options["league"],
            patch=options["patch"],
            defaults={"source": options["url"]},
        )
        if created:
            self.stdout.write(f"created data version {version}")
            return version

        if not options["force"]:
            raise CommandError(f"{version} already imported - pass --force to wipe and reimport")

        self.stdout.write(self.style.WARNING(f"clearing existing rows for {version}"))
        # SpawnWeight, GenerationWeight and m2m rows cascade with parents.
        Mod.objects.filter(data_version=version).delete()
        BaseItemType.objects.filter(data_version=version).delete()
        return version

    def import_base_items(self, data: dict, version: DataVersion) -> int:
        # Import released, named base items
        wanted = {
            metadata_id: entry
            for metadata_id, entry in data.items()
            if entry.get("release_state") == "released" and entry.get("name")
        }

        BaseItemType.objects.bulk_create(
            [
                BaseItemType(
                    data_version=version,
                    metadata_id=metadata_id,
                    name=entry["name"],
                    item_class=entry["item_class"],
                    domain=entry["domain"],
                )
                for metadata_id, entry in wanted.items()
            ],
            batch_size=2000,
        )

        base_ids = dict(
            BaseItemType.objects.filter(data_version=version).values_list("metadata_id", "id")
        )
 
        self.bulk_link(
            BaseItemType.tags.through,
            "baseitemtype_id",
            "tag_id",
            [
                (base_ids[metadata_id], self.tags[tag_name].id)
                for metadata_id, entry in wanted.items()
                for tag_name in entry.get("tags") or []
            ],
        )
 
        return len(wanted)
    @staticmethod
    def group_names(internal_id: str, entry: dict) -> list[str]:
        names = entry.get("groups") or ([entry["group"]] if entry.get("group") else [])
        return names or [internal_id]

    def import_mods(self, data: dict, version: DataVersion) -> int:
        seen = Counter(v["generation_type"] for v in data.values())
        self.stdout.write(f"generation types: {dict(seen.most_common())}")
 
        # mod rows 
        Mod.objects.bulk_create(
            [
                Mod(
                    data_version=version,
                    internal_id=internal_id,
                    name=entry.get("name", ""),
                    generation_type=entry["generation_type"],
                    domain=entry["domain"],
                    required_level=entry.get("required_level", 1),
                )
                for internal_id, entry in data.items()
            ],
            batch_size=2000,
        )
 
        mod_ids = dict(Mod.objects.filter(data_version=version).values_list("internal_id", "id"))
 
        # groups - collect every name then link
        groups_by_mod = {
            internal_id: self.group_names(internal_id, entry)
            for internal_id, entry in data.items()
        }
        all_group_names = {name for names in groups_by_mod.values() for name in names}
        group_ids = self.group_cache(all_group_names)
 
        self.bulk_link(
            Mod.groups.through,
            "mod_id",
            "modgroup_id",
            [
                (mod_ids[internal_id], group_ids[name])
                for internal_id, names in groups_by_mod.items()
                for name in names
            ],
        )
 
        # the mod tags
        self.bulk_link(
            Mod.tags.through,
            "mod_id",
            "tag_id",
            [
                (mod_ids[internal_id], self.tags[name].id)
                for internal_id, entry in data.items()
                for name in entry.get("implicit_tags") or []
            ],
        )
 
        # spawn weights - first matching tag wins
        SpawnWeight.objects.bulk_create(
            [
                SpawnWeight(
                    mod_id=mod_ids[internal_id],
                    tag=self.tags[sw["tag"]],
                    weight=sw["weight"],
                    order=position,
                )
                for internal_id, entry in data.items()
                for position, sw in enumerate(entry.get("spawn_weights") or [])
            ],
            batch_size=5000,
        )
 
        # generation weights e.g. fossil
        GenerationWeight.objects.bulk_create(
            [
                GenerationWeight(
                    mod_id=mod_ids[internal_id],
                    tag=self.tags[gw["tag"]],
                    value=gw["weight"],
                    order=position,
                )
                for internal_id, entry in data.items()
                for position, gw in enumerate(entry.get("generation_weights") or [])
            ],
            batch_size=5000,
        )
 
        return len(data)

    def import_added_tags(self, data: dict, version: DataVersion) -> int:
        mod_ids = dict(Mod.objects.filter(data_version=version).values_list("internal_id", "id"))
        pairs = [
            (mod_ids[internal_id], self.tags[name].id)
            for internal_id, entry in data.items()
            for name in entry.get("adds_tags") or []
        ]
 
        self.bulk_link(Mod.adds_tags.through, "mod_id", "tag_id", pairs)
 
        return len({mod_id for mod_id, _ in pairs})
