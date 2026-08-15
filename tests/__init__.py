from django.conf import settings


def test_settings_load():
    assert settings.SECRET_KEY
    assert settings.INSTALLED_APPS
