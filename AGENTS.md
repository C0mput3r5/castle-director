# AGENTS.md — Master Architecture & Game Design Blueprint
**Project:** *Castle Director* (3D Historical Colony-Sim & Indirect-Management Procedural Castle Builder)  
**Engine & Stack:** Rust + Bevy 0.15 ECS  
**Design Inspirations:** *Tiny Glade* (organic procedural staking/building), *Manor Lords* (authentic medieval resource logistics & settlement growth), *Majesty* (indirect rule, autonomous quirky heroes/workers, royal decrees).

---

## 1. Core Game Philosophy

### 1.1 Indirect Management ("The Director's Hand")
- **No God-Mode Instant Building:** The player never snaps finished walls into existence. Instead, the player acts as the **Director** who surveys the land, sketches **procedural blueprints** (rendered in-world as wooden stakes and taut twine/string), and issues high-level **Decrees, Priorities, & Subtle Whispers**.
- **Hierarchical Chain of Command:**
  - **The Director (Player):** Draws blueprint lines/zones, sets work priorities, issues decrees, and can Mind-Meld (`[V]`) into feudal lords to whisper subtle suggestions (`[Z][X][C]`).
  - **The Feudal Lords & Overseers (High Lord Aldric -> Vassal Lord Sir Barnaby -> Reeve -> Master Mason -> Royal Architect):** Autonomous authority figures with their own subjective worldview, patrols, and morale/speed command auras when supervising construction.
  - **The Workforce (Volunteer Peasants -> Woodcutters -> Quarriers -> Carpenters -> Masons -> Engineers):** Autonomous agents driven by a transparent state machine, personal quirks, stamina/hunger, and physical logistics.

### 1.2 Visible Construction Logistics
Every structure transitions through tangible, historically grounded physical stages:
1. **Staking & Surveying:** Wooden stakes driven into the turf connected by taut hemp guide strings.
2. **Earthworks & Foundations:** Digging ditches, piling earthen ramparts, hauling dirt baskets for a Motte mound, or trenching stone foundations.
3. **Raw Material Hauling:** Workers fell trees into physical timber logs, quarry stone, burn limestone in kilns for mortar, and haul materials to stockpiles or directly to blueprint stakes.
4. **Scaffolding & Erection:** Timber frames, wattle-and-daub panels, or timber palisade posts rise progressively out of the ground as workers hammer and fit materials into place.

### 1.3 Five Historical Eras of Castle Evolution
1. **Era 1 — The Humble Homestead (Ditch & Palisade):**
   - A rudimentary timber farmhouse, open-air log stockpile, turnip plots, and ditch-and-palisade timber defense erected by a Lead Farmer/Petty Lord and local volunteer peasants.
2. **Era 2 — Motte-and-Bailey:**
   - Earthwork mound (Motte) raised by dirt baskets and crowned by a wooden watchtower/keep, enclosed by an outer Bailey palisade, ditch, and timber gatehouse.
3. **Era 3 — Norman Stone Keep:**
   - Transition from timber to ashlar masonry. Quarries, lime kilns, wooden scaffolding, and a thick-walled square Norman Keep.
4. **Era 4 — Concentric Castle:**
   - Double curtain walls, round flanking towers, barbicans, murder holes, and specialized guild workshops led by a Master Mason.
5. **Era 5 — Grand Château / Late Medieval Fortress:**
   - High Gothic tracery, vaulted great halls, slate roofs, manicured courtyard gardens, and residential luxury fused with imposing fortifications.

### 1.4 Lighthearted & Quirky NPC Personalities
- Every NPC has a name, social role, and personality trait/quirk.
- Workers and Lords express thoughts, subjective opinions, and small complaints in a live **Peasant Activity Feed** and floating opinion tags (e.g., *"Giles is hauling a heavy oak log (complaining about his lower back)"*, *"Aldric the Farmer is admiring the straightness of the stakes"*).

---

## 2. MANDATORY DO'S AND DON'TS (Strict Engineering & Design Rules)

All AI agents and developers **MUST** follow these rules on every prompt without exception:

### 2.1 File Size & Modularity Rule
- **DO:** Keep every `.rs` source file clean, single-responsibility, and **strictly under 350 lines**.
- **DO:** Keep `src/main.rs` minimal—it must **ONLY** register plugins and global app state.
- **DON'T:** Create monolithic files or dump gameplay logic into `src/main.rs`. Split new features into focused ECS plugin modules when a file approaches 300–350 lines.

### 2.2 STL-Ready Solid Exterior Shell Rule
- **DO:** Generate every procedural castle piece (palisade posts, walls, towers, roofs, gatehouses, farmhouses) as a **closed, watertight (manifold) 3D solid volume** (e.g., closed cylinders, cuboids, cones, and solid extruded prisms) and tag it with an `ExteriorShell` component.
- **DO:** Tag the solid terrain base / meadow plinth with a `TerrainPlinth` component (using a closed 3D volume rather than a zero-thickness 2D plane).
- **DON'T:** Use paper-thin 2D planes (`Plane3d` without thickness) or hollow interior room shells for structures.
- **DON'T:** Tag transient entities (NPCs, trees, blueprint stakes, guide strings, loose logs, or carried props) with `ExteriorShell` or `TerrainPlinth`. When exporting to `.STL` on the backburner (`[F12]`), **ONLY** `ExteriorShell` and `TerrainPlinth` meshes will be exported into a watertight 3D-printable diorama.

### 2.3 No "God-Hand" Instant Building Rule
- **DO:** Route all player construction through the **Indirect Director Loop**:
  `Player Draws Blueprint Stake -> Autonomous NPC Hauls Resource -> Autonomous NPC Builds Piece`.
- **DON'T:** Allow the player to click-and-pop finished walls, towers, or buildings directly into existence without worker logistics.

### 2.4 Verification & Backlog Rule
- **DO:** Always run `cargo check` after editing any code and resolve all compiler errors and warnings before finishing a turn.
- **DO:** Update `BACKLOG.md` immediately whenever a checklist item or sub-task is completed.

---

## 3. Modular Bevy 0.15 ECS Plugin Architecture
- `state_plugin` (`src/state.rs`):
  - 5-Era Historical Evolution (`HistoricalEraState`: Era 1 Homestead -> Era 2 Motte-and-Bailey -> Era 3 Norman Stone Keep -> Era 4 Concentric Castle -> Era 5 Grand Château), automatic prestige unlocks, `[E]` manual era cycling, and Era 2 Motte / Era 3 Quarry & Kiln `ExteriorShell` landmarks.
- `camera_plugin` (`src/camera.rs`):
  - `TerrainPlinth` solid ground volume, Tabletop Grab-and-Slide panning (`Middle-Mouse` or `Left-Drag` in `[1]`), Zoom-to-Cursor with automatic horizon-to-top-down pitch tilting, Double-Click focus, and First-Person Eye Camera (`[V]`).
- `atmosphere_plugin` (`src/atmosphere.rs`):
  - Cozy Day/Dusk/Night/Dawn lighting cycle, 4-Season grass & foliage tinting (`GoldenSummer`, `CrispAutumn`, `CozyFrost`, `BloomingSpring`), `[T]` atmosphere toggle, and automatic warm flickering `CastleBrazierLight` lanterns on towers and gatehouses at night.
- `resources_plugin` (`src/world.rs`):
  - Central Homestead Farmhouse (`ExteriorShell`), Resource Stockpile zone with dynamic 3D log rack, harvestable perimeter Trees, and physical `GroundLog` items.
- `economy_plugin` (`src/economy.rs`):
  - Raised-bed Turnip Patch Plots (`ExteriorShell`), bubbling Homestead Stew Pot & Campfire, and `CozyNeeds` (`energy`, `stew_buff_timer`, `quirky_trait`) zero-death-spiral stew feast loop (`+50%` speed buff).
- `director_blueprint_plugin` & `procedural_construction_plugin` (`src/blueprint/mod.rs`, `src/blueprint/shell_mesh.rs`):
  - Context-sensitive Smart Brush (`[2]`) with live preview ribbon, bouncy `StakingPopAnimation`, Auto-Gate Carving across walls, Closed-Loop Auto-Tower Detection (`1–4` stories via Scroll/`PgUp`/`PgDn`), Right-Click Scrubber/Refund, Priority Bell (`[1]`), Two-Stage Ditch/Rampart & Scaffolding, swinging `GateDoor`s, and Era-aware timber/stone `ExteriorShell` curtain walls.
- `npc_ai_plugin` (`src/npc/mod.rs`, `src/npc/overseers.rs`):
  - High Lord Aldric & Vassal Lord Sir Barnaby (`LordMind`), specialist overseers (**Rowan the Reeve** & **Master Mason Cedric**), Reeve's Priority Decree (`[R]`), and Autonomous Peasant Workers.
- `pets_plugin` (`src/pets.rs`):
  - **Barnaby's Bane (The Loyal Castle Hound)** trotting behind High Lord Aldric, barking happily to cheer peasants, and jumping when petted with `[F]` in Mind-Meld mode, plus 4 wandering **Grazing Sheep**.
- `raiders_plugin` (`src/raiders.rs`):
  - Physical Homestead Warning Bell (`ExteriorShell`), Peaceful Sandbox default, and `[B]` Challenge Horn spawning slapstick Saucepan-Helmet **Turnip Bandits** pelted with flying turnips (`TossedTurnipProjectile`) who drop bonus logs & turnips without ever destroying player structures.
- `mind_meld_plugin` (`src/mind_meld.rs`):
  - First-Person `[V]` Mind-Meld mode, `[Tab]` Lord cycling, subjective mental visual filters & gizmos, floating 2D opinion tags, and `[Z][X][C]` Whisper Inception mechanics.
- `stl_export_plugin` (`src/stl_export.rs`):
  - Pure-Rust Binary `.STL` 3D-printer exporter (`[F12]`) that bakes world-transformed triangles exclusively from `ExteriorShell` and `TerrainPlinth` meshes into `exported_castle_diorama.stl` (verified by unit tests).
- `ui_hud_plugin` (`src/ui.rs`):
  - Parchment-and-dark-wood "Castle Chronicle Tapestry" HUD, collapsible `[H]` Royal Director Cheat-Sheet Help Card, celebratory `.STL` export banner, Mind-Meld overlay, and interactive command buttons.
