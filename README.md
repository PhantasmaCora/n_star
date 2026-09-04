A personal roguelike project I'm developing using bracket-lib for Rust. (Find bracket-lib here: https://github.com/amethyst/bracket-lib/tree/master)

Current features include:
+ Map generation infrastructure (Presently coded to use a fixed seed for debug purposes).
+ Actors with a control flow for selecting and executing actions. Moving around the map.
+ Simple NPC brains based on "wander" and "don't stray too far from my packmates" behaviors.
+ Items, inventory, capacity limits, picking up and dropping.
+ Field-of-view and memory for what tiles the player sees.

Planned but not yet implemented features include:
- Inspecting objects in inventory or environment for more information. Usable or consumable items.
- "Attachments" - a combination of body plan and equipment systems. Tree structure, like "body provides arms slot, arms provide weapon slot."
- Combat, health, damage; micro-HP and macro-Wounds system.
- More sophisticated NPC brains.
- Deliberate system for populating maps with NPCs and loot.
- A 'main menu' and character creation.
- Movement down the levels (planned to have about sixteen floors in the dungeon, with scaling difficulty and biomes randomly selected from a list).
- Reading in various content from data files rather than hardcoding.

As this code is my personal prototype for learning and enjoyment, janky architecture and implementations are to be expected; I make this repo visible mainly because some friends asked to take a look.
No LLMs or other statistical-regurgitation technology were used in the creation of this code, nor will they ever be.
