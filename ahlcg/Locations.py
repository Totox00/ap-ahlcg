from typing import Optional, Callable

from BaseClasses import Location, CollectionState

from .Data import campaigns, scenarios

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
        groups = {
            **{c: set() for c in campaigns},
            **{s: set() for s in scenarios}
        }

        for scenario in scenarios.values():
            s_group = groups[scenario.name]
            for location in scenario.locations:
                for i in range(0, location.clues):
                    s_group.add(f"{scenario.name} - {location.name} Clues {i + 1}")
                for i in range(0, location.victory):
                    s_group.add(f"{scenario.name} - {location.name} Victory {i + 1}")
            for check in scenario.checks:
                s_group.add(check.name)
            groups[scenario.campaign].update(s_group)

        return groups
