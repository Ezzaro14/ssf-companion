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

## Wishlist
- **Strategy analysis** - provide the tool what you need to farm and it will 
  suggest an optimized farming strategy.
- **Map Data & Layout Analysis** - Suggestion for Map layouts depending on 
  strategy. Open maps for breaches, "Linear" maps for delirium, etc.

## Why I made this

I'm a big fan of Path of Exile. I have thousands of hours, but I've never been
"good" at the game. In softcore, I've only recently been able to amass 100 divs
worth of currency. I'm a very casual gamer where it takes me a month to get my 4
watchstones.

This also means that while I love playing SSF, I struggle a lot. I'm not great at
crafting, if I need a certain body armour, I'm not sure if I should target farm div
cards or something else. 

I'm making this as a way to get better at coding and as a nice guide for myself on
how to proceed in the most efficient way while being VERY inefficient in the game myself
(I do maybe 6 maps an hour?).

This tool will help figure out best crafting based on what I run and how much loot of each
currency I drop per hour and possibly help me be more efficient when I add deeper strategy
analysis which could help me find a strategy. For example, if my build needs a cluster jewel,
a foulborn unique armour, and a ritual corpse, it will help me make an atlas passive map that 
has delirium, breach, and rituals. Maybe I'm also low on maps so it will allocate destructive play,
or I will need scarabs so it will focus on scarab nodes, or both!

That is my plan for this. Will it work? I highly doubt it to be honest. But I will try.

Thanks for reading! I'm in the early stages of this right now but hopefully
once it's somewhat functional, you will find it useful for yourself <3

- Ezzar 06/09/2026

## Attribution

Modifier and base item data from
[RePoE](https://github.com/lvlvllvlvllvlvl/RePoE). Not affiliated with or
endorsed by Grinding Gear Games.

Licensed under MIT OR Apache-2.0.
