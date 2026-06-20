from dataclasses import dataclass

from schema import Schema, And, Optional, Or

from Options import Toggle, Range, Choice, PerGameCommonOptions, ItemSet, OptionDict, OptionSet, OptionGroup, OptionList
from worlds.ahlcg.Data import campaigns, scenarios


class IncludedCampaigns(OptionSet):
    """All campaigns that will be included"""
    display_name = "Included Campaigns"
    default = frozenset(campaigns.keys())
    valid_keys = campaigns.keys()


class RequiredCampaigns(Range):
    """The number of campaigns where the final scenario must be completed in order to goal"""
    display_name = "Required Campaigns"
    range_start = 0
    range_end = len(campaigns)
    default = 1


class StartingScenarios(Range):
    """The number of scenarios that will start unlocked"""
    display_name = "Starting Scenarios"
    range_start = 0
    range_end = [scenario.logic_xp for scenario in scenarios.values()].count(0)
    default = 1


class XpLogicModifier(Range):
    """The value all logically expected xp for scenarios will be multiplied with, as percent"""
    display_name = "XP Logic Modifier"
    range_start = 0
    range_end = 200
    default = 100


class Difficulty(Choice):
    """The difficulty used for campaigns. This affects which filler and trap items exist in the pool and is intended to be matched to the difficulty selected for each campaign."""
    display_name = "Difficulty"
    option_easy = 0
    option_standard = 1
    option_hard = 2
    option_expert = 3
    default = 1


@dataclass
class AhlcgOptions(PerGameCommonOptions):
    included_campaigns: IncludedCampaigns
    required_campaigns: RequiredCampaigns
    starting_scenarios: StartingScenarios
    xp_logic_modifier: XpLogicModifier
    difficulty: Difficulty


ahlcg_option_groups = [
    OptionGroup("Basic", [IncludedCampaigns, RequiredCampaigns, StartingScenarios, XpLogicModifier, Difficulty]),
]
