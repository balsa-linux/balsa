# Calamares injects this module from C++, so it only exists inside the installer.

from typing import Any

class _GlobalStorage:
    def value(self, key: str) -> Any: ...
    def insert(self, key: str, value: Any) -> None: ...

class _Utils:
    def debug(self, message: str) -> None: ...
    def warning(self, message: str) -> None: ...

globalstorage: _GlobalStorage
utils: _Utils
