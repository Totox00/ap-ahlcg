# This file is generated as part of the compilation of the client
from typing import NamedTuple, Callable, Set, Optional, Iterable
from BaseClasses import CollectionState
from collections import Counter
class Filler(NamedTuple):
    name:str
    trap:int
    quantity:list[int]
class Campaign(NamedTuple):
    name:str
    xp:int
    scenarios:list[str]
    unlocks:list[str]
    scenario_cards:list[str]
    filler:list[Filler]
class Location(NamedTuple):
    name:str
    clues:int
    victory:int
class Path(NamedTuple):
    origin:str
    destination:str
    rule:Callable[[CollectionState,int],bool]
class Check(NamedTuple):
    name:str
    goal:int
    rule:Callable[[CollectionState,int],bool]
class Scenario(NamedTuple):
    name:str
    campaign:str
    logic_xp:int
    locations:list[Location]
    begin:list[str]
    paths:list[Path]
    checks:list[Check]
