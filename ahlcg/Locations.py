from typing import Optional, Callable

from BaseClasses import Location, CollectionState


class AhlcgLocation(Location):
    game: str = "Arkham Horror The Card Game"

    def __init__(
            self,
            player: int,
            name: str,
            address: Optional[int],
            parent,
            rule: Optional[Callable[[CollectionState, int], bool]] = None):
        super().__init__(player, name, address, parent)
        if rule is not None:
            self.access_rule = lambda state: rule(state, player)

    @staticmethod
    def get_location_name_groups() -> dict:
        return {}
