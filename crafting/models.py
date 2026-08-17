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

