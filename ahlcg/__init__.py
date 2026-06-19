import math
from itertools import chain
from typing import Dict, Set, Optional, NamedTuple, ClassVar

from BaseClasses import MultiWorld, Region, Item, Tutorial, ItemClassification, CollectionState
from Options import Toggle, OptionError

from worlds.AutoWorld import World, WebWorld

from .Items import AhlcgItem
from .Locations import AhlcgLocation
from .Options import AhlcgOptions, ahlcg_option_groups
from .Data import campaigns, scenarios
from .Id import item_name_to_id, location_name_to_id


class AhlcgWeb(WebWorld):
    bug_report_page = "https://github.com/Totox00/ap-ahlcg/issues"
    theme = "ocean"
    option_groups = ahlcg_option_groups
    tutorials = [Tutorial(
        "Multiworld Setup Guide",
        "A guide to setting up the Archipelago Arkham Horror The Card Game randomizer client on your computer.",
        "English",
        "setup_en.md",
        "setup/en",
        ["Toto00"]
    )]


class AhlcgWorld(World):
    """
    The boundaries between worlds have drawn perilously thin...
    Arkham Horror: The Card Game is a cooperative card game set amid a backdrop of Lovecraftian horror. As the Ancient Ones seek entry to our world, one to four investigators work to unravel arcane mysteries and conspiracies.
    Their efforts determine not only the course of your game, but carry forward throughout whole campaigns, challenging them to overcome their personal demons even as Arkham Horror: The Card Game blurs the distinction between the card game and roleplaying experiences.
    ― Fantasy Flight Games
    """

    game = "Arkham Horror The Card Game"
    options_dataclass = AhlcgOptions
    options: AhlcgOptions
    topology_present: bool = False
    web = AhlcgWeb()

    included_campaigns: Set[str]
    required_campaigns: int
    starting_scenarios: int
    xp_logic_modifier: float
    difficulty: int

    total_locations: int
    total_items: int

    required_client_version = (0, 0, 1)
    item_name_to_id = item_name_to_id
    location_name_to_id = location_name_to_id
    item_name_groups = AhlcgItem.get_item_name_groups(item_name_to_id)
    location_name_groups = AhlcgLocation.get_location_name_groups()

    def __init__(self, multiworld: MultiWorld, player: int):
        super().__init__(multiworld, player)
        self.total_items = 0
        self.total_locations = 0

    def generate_early(self):
        self.included_campaigns = self.options.included_campaigns.value
        self.required_campaigns = self.options.required_campaigns.value
        self.starting_scenarios = self.options.starting_scenarios.value
        self.xp_logic_modifier = self.options.xp_logic_modifier.value / 100
        self.difficulty = self.options.difficulty.value

        choices = [scenario for scenario in scenarios.values() if scenario.campaign in self.included_campaigns and scenario.logic_xp == 0]
        for _ in range(0, self.starting_scenarios):
            choice = self.random.choice(choices)
            choices.remove(choice)
            name = f"{choice.campaign} - {choice.name}"
            self.multiworld.push_precollected(AhlcgItem(self.player, name, item_name_to_id[name], ItemClassification.progression))

    def create_regions(self):
        menu = Region("Menu", self.player, self.multiworld)
        self.multiworld.regions.append(menu)

        for scenario in scenarios.values():
            if scenario.campaign in self.included_campaigns:
                scenario_region = Region(scenario.name, self.player, self.multiworld)
                self.multiworld.regions.append(scenario_region)
                menu.connect(scenario_region, f"Menu -> {scenario.name}", lambda state, scenario=scenario: state.has(f"{scenario.campaign} - {scenario.name}", self.player) and state.has(f"{scenario.campaign} XP", self.player, self.xp_logic_modifier * scenario.logic_xp))
                regions = {}
                for location in scenario.locations:
                    region = Region(location.name, self.player, self.multiworld)
                    regions[location.name] = region
                    for i in range(0, location.clues):
                        name = f"{scenario.name} - {location.name} Clues {i + 1}"
                        region.locations.append(AhlcgLocation(self.player, name, location_name_to_id[name], region))
                        self.total_locations += 1
                    for i in range(0, location.victory):
                        name = f"{scenario.name} - {location.name} Victory {i + 1}"
                        region.locations.append(AhlcgLocation(self.player, name, location_name_to_id[name], region))
                        self.total_locations += 1
                self.multiworld.regions.extend(regions.values())
                for location in scenario.begin:
                    scenario_region.connect(regions.get(location), f"Menu -> {location}", lambda state: True)
                for path in scenario.paths:
                    regions.get(path.origin).connect(regions.get(path.destination), f"{path.origin} -> {path.destination}", lambda state, path=path: path.rule(state, self.player))
                for check in scenario.checks:
                    scenario_region.locations.append(AhlcgLocation(self.player, check.name, location_name_to_id[check.name], scenario_region, check.rule))
                    self.total_locations += 1
                    if check.goal:
                        event_location = AhlcgLocation(self.player, f"{check.name} - Goal campaign", None, scenario_region, check.rule)
                        event_location.place_locked_item(AhlcgItem(self.player, f"{scenario.campaign} - Goal campaign", None, ItemClassification.progression))
                        scenario_region.locations.append(event_location)

    def create_items(self):
        exclude = [item.name for item in self.multiworld.precollected_items[self.player]]
        items = []

        for campaign in campaigns.values():
            if campaign.name in self.included_campaigns:
                for item in chain(campaign.scenarios, campaign.unlocks, campaign.scenario_cards):
                    name = f"{campaign.name} - {item}"
                    if name not in exclude:
                        items.append(AhlcgItem(self.player, name, item_name_to_id[name], ItemClassification.progression))
                for _ in range(0, campaign.xp):
                    name = f"{campaign.name} - 1 XP"
                    items.append(AhlcgItem(self.player, name, self.item_name_to_id[name], ItemClassification.progression_deprioritized, campaign.name, 1))
                for filler in campaign.filler:
                    name = f"{campaign.name} - {filler.name}"
                    code = self.item_name_to_id[name]
                    classification = ItemClassification.trap if filler.trap & (1 << self.difficulty) > 0 else ItemClassification.filler
                    for _ in range(0, filler.quantity[self.difficulty]):
                        items.append(AhlcgItem(self.player, name, code, classification))
        
        choices = [campaign for campaign in campaigns.values() if campaign.name in self.included_campaigns]
        while self.total_locations > len(items):
            campaign = self.random.choice(choices)
            name = f"{campaign.name} - 1 XP"
            items.append(AhlcgItem(self.player, name, self.item_name_to_id[name], ItemClassification.useful, campaign.name, 1))

        self.multiworld.itempool.extend(items)

    def create_item(self, name: str) -> Item:
        id = self.item_name_to_id.get(name, None)
        if id is None:
            raise OptionError(f"Item {name} does not exist")
        return AhlcgItem(self.player, name, id, ItemClassification.progression_deprioritized)

    def get_filler_item_name(self) -> str:
        choices = [campaign for campaign in campaigns.values() if campaign.name in self.included_campaigns]
        return f"{self.random.choice(choices)} - 1"

    def set_rules(self) -> None:
        self.multiworld.completion_condition[self.player] = lambda state: [state.has(f"{campaign} - Goal campaign", self.player) for campaign in self.included_campaigns].count(True) >= self.required_campaigns

    def collect(self, state: CollectionState, item: AhlcgItem):
        changed = super().collect(state, item)
        if item.campaign:
            state.prog_items[self.player][f"{item.campaign} XP"] += item.xp
            if item.xp > 0:
                changed = True
        return changed

    def remove(self, state: CollectionState, item: AhlcgItem):
        changed = super().remove(state, item)
        if item.campaign:
            state.prog_items[self.player][f"{item.campaign} XP"] -= item.xp
            if item.xp > 0:
                changed = True
        return changed

    def fill_slot_data(self) -> Dict[str, object]:
        return {
            "g": self.required_campaigns
        }
