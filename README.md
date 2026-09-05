# SSF Companion

[![ci](https://github.com/Ezzaro14/ssf-companion/actions/workflows/ci.yml/badge.svg)](https://github.com/Ezzaro14/ssf-companion/actions/workflows/ci.yml)

A Path of Exile crafting planner for SSF. The idea is that instead of looking at prices of currency,
SSF Companion prices every decision in **farm time**. It will check what currency you have and what 
currency you don't, and find the crafting steps most suitable for you.

## Why this is a rewrite

The first version of this was a Django web app
([archived here](https://github.com/Ezzaro14/ssf-companion-django)). The
search over crafting routes turned out to be a long CPU-bound search over 
hundreds of thousands of states, not a query. Hosting it would have also been
a horrible idea.

So it is now a native desktop app with the solver in Rust!

## Status

- Toolchain & Workspace Created

## The crate graph

Dependencies only ever point downward in this list.

    ssf-data      RePoE ingest, the GameData index, the binary snapshot
    ssf-economy   currencies, rates: what an orb costs in minutes
    ssf-craft     item state, odds, and the crafting mechanics
    ssf-solve     value iteration. knows nothing about the game
    ssf-plan      policy to procedure: rows, the bill, the simulation
    ssf-cli       a development binary, usable with no UI
    xtask         build the snapshot from the dump

## Planned

- **Probability engine** — exact odds per mechanic, and a simulation of the
  chosen policy for the median and the unlucky case
- **Route search** — compares currency, essences, fossils, bench crafts,
  beast recipes and meta-crafts, and returns the cheapest route to the target
- **Yield tracking** — records what you actually find, per activity, and
  feeds those rates back into the planner instead of the shipped baseline
- **Item parsing** — paste an item from the game and resolve its pools
- **Build gap analysis** — import a Path of Building code to see what you need
  to craft

## Attribution

Modifier and base item data from
[RePoE](https://github.com/lvlvllvlvllvlvl/RePoE). Not affiliated with or
endorsed by Grinding Gear Games.

Licensed under MIT OR Apache-2.0.
