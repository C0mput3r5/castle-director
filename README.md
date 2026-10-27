# Castle Director 🏰

**3D Historical Colony-Sim & Indirect-Management Procedural Castle Builder**  
Built with **Rust + Bevy 0.15 ECS**  
*Inspired by Tiny Glade, Manor Lords, and Majesty*

---

## Overview

In **Castle Director**, you are the **Unseen Guiding Hand** behind a living medieval homestead that evolves across **5 Historical Eras** (*The Humble Homestead -> Motte-and-Bailey -> Norman Stone Keep -> Concentric Castle -> Grand Château*).

- **Autonomous NPC Architects:** **High Lord Aldric** (*Practical Farmer Lord*), **Sir Barnaby** (*Paranoid Martial Vassal*), and **Master Mason Cedric** autonomously survey, debate, stake out, and construct a concentric castle ring-by-ring around the homestead.
- **The Unseen Whim (`[2]`):** Whenever you add stakes, drag across a wall to carve a **Gatehouse Archway**, close a loop for a **Multi-Story Watchtower**, or `Right-Click` to veto/erase stakes into an open courtyard, your servants have zero awareness of an outside player—they treat every change as their own stroke of genius and immediately abide without question.
- **Watertight `.STL` 3D-Printer Exporter (`[F12]`):** Every permanent architectural element (`ExteriorShell`) and the ground base (`TerrainPlinth`) is generated as a closed, manifold 3D solid ready to export to `exported_castle_diorama.stl` for 3D printing.

---

## Quick-Start Instructions (Running on a Second PC)

### 1. Install Rust (`rustup`)
If Rust is not yet installed on the machine:
```powershell
winget install Rustlang.Rustup
```
*(Or download from [https://rustup.rs](https://rustup.rs) and restart your terminal so `cargo` is on your `PATH`.)*

### 2. Clone & Run
```bash
git clone https://github.com/C0mput3r5/castle-director.git
cd castle-director
cargo run
```
> **Tip:** `Cargo.toml` is pre-configured with `opt-level = 1` in dev and `opt-level = 3` for dependencies so Bevy runs smoothly right out of the box.

### 3. Verify Build & Unit Tests (Without Launching Window)
```bash
cargo check --all-targets
cargo test
```

---

## Controls Cheat-Sheet

| Key / Mouse | Action |
| :--- | :--- |
| **`[Space]`** | **Sound Work Horn** (Start from Council Briefing) / **Pause & Resume Work** |
| **`[1]`** | **Observe & Inspect Mode (Default):** Hover over any Villager, Lord, Stake, or Building to inspect; `Left-Drag` to pan camera; `Left-Click` an unbuilt stake to ring its **Priority Bell (2.5x speed)**; `Left-Click` a debating Lord to break a tie |
| **`[2]`** | **Subconscious Whim Mode:** `Left-Drag` to sketch stakes, drag across a wall to carve a **Gatehouse Archway** in-place, close a loop for a **Watchtower Keep**, or drag over wooden walls in Era 3+ to retrofit to **Stone**; `Right-Drag` to **Veto/Erase** (NPCs will never re-stake vetoed courtyards) |
| **`[G]`** | Have Lords & Master Mason Cedric propose the next **Concentric Ring Blueprint** immediately (or resolve an active debate) |
| **`[Z]` / `[X]`** | **Break Feudal Debate Tie** (Back High Lord Aldric `[Z]` or Sir Barnaby `[X]`) — or issue **Whisper Inceptions** (`[Z]` Urgency, `[X]` Praise, `[C]` Rivalry) while inside a Lord's mind (`[V]`) |
| **`[V]` / `[Tab]`** | Toggle **First-Person Mind-Meld** into a Lord's eyes (`[Tab]` switches between High Lord Aldric & Sir Barnaby) |
| **`[F]`** | **Pet Barnaby's Bane** (the Loyal Castle Hound) while in `[V]` Mind-Meld mode |
| **`[R]`** | Cycle **Rowan the Reeve's Priority Decree** (*Balanced*, *Timber & Quarrying*, *Castle Construction*, *Campfire Feast*) |
| **`[3]`** | Summon an extra **Volunteer Peasant** from the Homestead Farmhouse |
| **`[E]`** | Manually cycle **Historical Era (Eras 1–5)** (also advances automatically as objectives complete) |
| **`[T]`** | Cycle **Cozy Time of Day & 4 Seasons** (*Golden Summer*, *Crisp Autumn*, *Cozy Frost*, *Blooming Spring*) |
| **`[B]`** | Blow the **Challenge Horn** for a non-destructive slapstick **Turnip Bandit** skirmish |
| **`[H]` / `[Tab]`** | Toggle the **Royal Director's Briefing & Era Objective Charter** |
| **`[F12]`** | Export watertight 3D-printable castle diorama to **`exported_castle_diorama.stl`** |
| **Camera** | `WASD` / `Middle-Drag` (or `Left-Drag` in `[1]`) Pan, `Right-Drag` / `Q`/`E` Orbit, `Scroll` Zoom-to-Cursor, `Double-Click` Focus, `PgUp`/`PgDn` (or `Scroll` on Tower Handle) Adjust Tower Stories (`1–4`) |

---

## Project Architecture & Governance
- **[AGENTS.md](./AGENTS.md):** Core game philosophy, strict modularity rules (`< 350` lines per `.rs` file), and watertight `ExteriorShell` 3D-printing standards.
- **[BACKLOG.md](./BACKLOG.md):** Live development checklist and completed milestones.

