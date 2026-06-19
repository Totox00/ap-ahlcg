from typing import Optional

from BaseClasses import Item, ItemClassification


class AhlcgItem(Item):
    game: str = "Arkham Horror The Card Game"

    def __init__(
            self,
            player: int,
            name: str,
            code: Optional[int],
            classification: ItemClassification,
            campaign: Optional[str] = None,
            xp: int = 0):
        super().__init__(name, classification, code, player)
        self.campaign = campaign
        self.xp = xp

    @staticmethod
    def get_item_name_groups(item_name_to_id: dict[str, int]) -> dict:
        return {}
