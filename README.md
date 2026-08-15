![CI](https://github.com/Ezzaro14/ssf-companion/actions/workflows/ci.yml/badge.svg)
# SSF Companion

A crafting planner for Path of Exile SSF

## Features

- **Item parsing** -
  paste an item straight from the game and it resolves the modifier pools that item can roll from
- **Probability engine** -
  exact odds for single-modifier outcomes, Monte Carlo simulation for multi-step sequences
- **Route search** -
  compares crafting possibilities (currency, essences, fossils, bench crafts, beast recipes, meta-crafts) and returns the cheapest route to the target
- **Yield tracking** -
  records what you actually find, per activity, and feeds those rates back into the planner
- **Build gap analysis** -
  import a Path of Building code and see what you're still missing

## Status

**Early development (WIP)**
- Repo Bootstap - Django, DRF, Postgres, CI, py tests, env configs
- Copied Item Text Parser - structures and reads copied item data  

**Plans**
- Crafting Probability 
- Possible Crafting Route Advice
- Required Item Tracker
- Item Acquisition Guide
- Currency Drop Data from your sessions(Zone Tracking)
- PoB Import?


## Local setup

```bash
git clone https://github.com/Ezzaro14/ssf-companion.git
cd ssf-companion

python -m venv .venv
source .venv/bin/activate        # Windows: .\.venv\Scripts\Activate.ps1
pip install -r requirements-dev.txt

cp .env.example .env             # then set SECRET_KEY
docker compose up -d             # starts postgres

python manage.py migrate
python manage.py runserver
pytest
```

---

*Not affiliated with or endorsed by Grinding Gear Games*
