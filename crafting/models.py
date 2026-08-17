from django.db import models

# Create your models here.
class DataVersion(models.Model):
    """One import run. Every BaseItemType and Mod belongs to exactly one."""

    league = models.CharField(max_length=100)      # "Settlers", "Standard"
    patch = models.CharField(max_length=50)        # "3.25.0"
    source = models.CharField(max_length=500)      # URL the dump came from
    imported_at = models.DateTimeField(auto_now_add=True)
    is_active = models.BooleanField(default=False)  # which version queries default to

    class Meta:
        unique_together = [("league", "patch")]  # one import per league+patch pair

    def __str__(self):
        return f"{self.league} {self.patch}"

class Tag(models.Model):
    """Game data tag - "flask", "utility_flask", "weapon". Mod eligibility is decided by tags."""

    name = models.CharField(max_length=100, unique=True)

    def __str__(self):
        return self.name


class BaseItemType(models.Model):
    data_version = models.ForeignKey(DataVersion, on_delete=models.CASCADE, related_name="base_items")
    name = models.CharField(max_length=200)         # "Basalt Flask"
    item_class = models.CharField(max_length=100)   # "UtilityFlask"
    domain = models.CharField(max_length=50)        # only same-domain mods can roll here
    tags = models.ManyToManyField(Tag, related_name="base_item_types")

    class Meta:
        unique_together = [("data_version", "name")]
        indexes = [models.Index(fields=["data_version", "domain"])]

    def __str__(self):
        return self.name

class ModGroup(models.Model):
    """Mods sharing a group cannot coexist on one item - stops three "increased Life" prefixes."""

    name = models.CharField(max_length=200, unique=True)

    def __str__(self):
        return self.name


class Mod(models.Model):
    PREFIX = "prefix"
    SUFFIX = "suffix"
    GENERATION_TYPES = [
        (PREFIX, "Prefix"),
        (SUFFIX, "Suffix"),
        ("corrupted", "Corrupted"),
        ("enchantment", "Enchantment"),
        ("unique", "Unique"),
    ]

    data_version = models.ForeignKey(DataVersion, on_delete=models.CASCADE, related_name="mods")
    internal_id = models.CharField(max_length=200)   # "FlaskIncreasedMovementSpeed3"
    name = models.CharField(max_length=200)          # "of Adrenaline" - what AMD shows
    generation_type = models.CharField(max_length=30, choices=GENERATION_TYPES)
    group = models.ForeignKey(ModGroup, on_delete=models.PROTECT, related_name="mods")
    domain = models.CharField(max_length=50)
    required_level = models.PositiveIntegerField(default=1)  # ilvl gate
    adds_tags = models.ManyToManyField(Tag, related_name="added_by_mods", blank=True)

    class Meta:
        unique_together = [("data_version", "internal_id")]
        indexes = [models.Index(fields=["data_version", "domain", "generation_type"])]

    def __str__(self):
        return f"{self.internal_id} ({self.name})"
