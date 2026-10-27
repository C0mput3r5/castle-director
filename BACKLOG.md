# BACKLOG.md — Live Development Checklist & Roadmap

**Project:** *Castle Director* (Rust + Bevy 0.15)  
**Governance:** Always keep this file updated as features are implemented and verified with `cargo check`.

---

## Milestone 1: Phase 1 Homestead Core
- [x] **3D World & RTS Director Camera (`src/camera.rs`):** Sunlit 3D meadow ground plinth (`TerrainPlinth`), directional shadows, warm ambient lighting, and smooth camera controls (`WASD` pan, `Right-Click + Drag` / `Q`/`E` orbit, `Mouse Scroll` zoom).
- [x] **Procedural Meadow, Farmhouse & Trees (`src/world.rs`):** Central rudimentary Farmhouse (`ExteriorShell`), adjacent Resource Stockpile zone with dynamic 3D log rack, and 28 procedurally scattered harvestable perimeter Trees that drop physical Timber Logs on the ground.
- [x] **Tiny-Glade Style Stake Blueprint Drawing (`src/blueprint.rs`):** Ground-plane raycasting on `Left-Click + Drag` spawning Wooden Stakes connected by taut guide strings every `~1.5` units, creating `ConstructionSite` entities requiring `1 Timber Log`.
- [x] **Lead Farmer + 4 Autonomous Peasants (`src/npc.rs`):** Lead Farmer ("Aldric the Petty Lord-to-be") with golden crown marker and `+55%` worker speed inspection aura; 4 quirky named Peasant Workers autonomously chopping trees, hauling visible 3D timber logs, and hammering staked slots into rising watertight Wooden Palisade Posts (`ExteriorShell`).
- [x] **Visual Testing HUD (`src/ui.rs`):** Top Era 1 banner, live resource/blueprint/post counters, interactive buttons & hotkeys (`[1]` Inspect, `[2]` Draw Palisade Blueprint, `[3]` Spawn Extra Peasant, `[Space]` 1x/3x Speed), and bottom-left live Peasant Activity Feed.

---

## Milestone 1.5: Feudal Hierarchy & First-Person Mind-Meld
- [x] **Feudal Lord Hierarchy (`src/npc.rs`):** High Lord Aldric (`PracticalFarmerLord`, gold crown & royal cape) + Visiting Vassal Lord Sir Barnaby (`ParanoidMartialVassal`, iron/crimson crest) equipped with `LordMind` components (`name`, `title`, `archetype`, live `current_thought`, `ego`, `whisper_influence`, and quirky `opinions`).
- [x] **`[V]` First-Person Eye Camera (`src/camera.rs`, `src/mind_meld.rs`):** Seamless toggle between `CameraMode::DirectorRTS` and `CameraMode::FirstPersonLord(Entity)` locked to the possessed Lord's eye level (`+1.7Y`) with mouse/keyboard look-around offsets and automatic hiding/restoring of the possessed Lord's mesh.
- [x] **`[Tab]` Lord Cycling (`src/mind_meld.rs`):** Cycle active First-Person Mind-Meld consciousness between High Lord Aldric and Vassal Lord Sir Barnaby.
- [x] **Subjective Mental Visual Filters (`src/mind_meld.rs`):** Dynamic ambient lighting & sky tinting plus 3D Gizmo overlays per Lord archetype (Aldric's warm golden Farmer Filter with green timber/stockpile rings; Sir Barnaby's tense blue/red Paranoid Martial Filter with pulsing Red Warning Gizmos on unfinished stakes).
- [x] **Floating Opinion Tags (`src/mind_meld.rs`):** World-to-viewport projected 2D UI opinion tags floating above nearby Peasants and Lords in front of the possessed Lord.
- [x] **`[Z][X][C]` Whisper Inception Mechanics (`src/mind_meld.rs`, `src/ui.rs`, `src/npc.rs`):** Dedicated Mind-Meld HUD banner, live Inner Monologue box, and 3 Whisper hotkeys (`[Z]` Whisper Urgency -> 2x build speed aura & sprint to stakes; `[X]` Whisper Praise -> warm turnip stew morale & tree-chopping boost; `[C]` Whisper Rivalry -> feudal debate & 2x aura radius).

---

## Autonomous Sprint — Step 1 of 5: Tactile "Tiny Glade" Camera & Smart Brush Controls
- [x] **Tabletop "Grab & Slide" Camera with Smooth Damping (`src/camera.rs`):** Silky `lerp` damping on focus, yaw, pitch, and radius; Grab-the-World raycast panning (`Middle-Mouse` or `Left-Drag` in Inspect Mode `[1]`); Zoom-to-Cursor & automatic horizon-to-top-down pitch tilting; Double-Click entity/structure framing.
- [x] **Context-Sensitive "Smart Brush" Blueprint Tool (`src/blueprint.rs`, `src/construction.rs`):** Live ground preview ribbon while dragging in Build Mode `[2]`; bouncy elastic scale-in `StakingPopAnimation`; **Auto-Gate Carving** (`SiteKind::GatehouseArchway`) when crossing existing stakes/walls; **Closed-Loop Auto-Tower Detection** (`TowerFoundation` + central height-handle gizmo adjustable 1–4 stories via Scroll or `PageUp`/`PageDown`); **Right-Click Scrubber / Eraser** that dismantles stakes/walls and refunds physical `GroundLog` timber onto the ground.
- [x] **Priority Bell ("Golden Acorn" Nudge) (`src/blueprint.rs`, `src/construction.rs`, `src/npc.rs`):** Left-clicking any unbuilt stake or tower in Inspect Mode `[1]` rings the Priority Bell (pulsing golden gizmo rings), triggering a joyful peasant hop and 2.5x priority build speed.

---

## Milestone 2 / Autonomous Sprint — Step 2 of 5: Procedural "ExteriorShell" Architecture, Ditches & Historical Era Evolution
- [x] **Connected Curved Palisade & Wall Segments (`src/blueprint/shell_mesh.rs`):** Smoothly joined palisade curtains (vertical logs + horizontal timber bracing in Era 1–2) upgrading to thick bevelled stone masonry courses with battlements/crenellations in Era 3–5.
- [x] **Dug-Out Dirt Ditches & Raised Earthen Ramparts (`src/blueprint/shell_mesh.rs`, `src/npc.rs`):** Two-stage construction where Stage 1 digs a depressed trench marker in front and a raised sloped earthen rampart mound behind the wall (`ExteriorShell`), and Stage 2 erects the wall/tower.
- [x] **Functional Gatehouse & Multi-Story Towers (`src/blueprint/shell_mesh.rs`):** Gatehouse slots build two flanking posts, a solid overhead arch/walkway, and wooden doors (`GateDoor`) that swing open when friendly NPCs pass within 3 units; closed-loop Tower blueprints build multi-story keeps topped with conical roofs (`ExteriorShell`).
- [x] **Procedural Wooden Scaffolding (`src/blueprint/shell_mesh.rs`):** Temporary wooden scaffolding poles and planks (`TemporaryProp`, never `ExteriorShell`) spawn around towers/gatehouses/stone walls under construction and vanish in a puff (`ScaffoldingPuff`) upon completion.
- [x] **5-Era Historical Evolution System (`src/state.rs`, `src/ui.rs`):** `HistoricalEraState` (`Era1Homestead` -> `Era2MotteAndBailey` -> `Era3NormanStoneKeep` -> `Era4ConcentricCastle` -> `Era5GrandChateau`) that advances automatically with castle prestige or anytime via HUD button / hotkey `[E]`, spawning the Era 2 Motte Earthwork Mound & Timber Watchtower and Era 3 Stone Quarry & Lime Kiln (`ExteriorShell`).
- [x] **Watertight `ExteriorShell` & `TerrainPlinth` Tagging:** Every permanent structure piece and ground plinth is tagged and constructed as a closed, manifold 3D solid volume.

---

## Milestone 3 / Autonomous Sprint — Step 3 of 5: Cozy Economy, Automated Guilds, Chain of Command & Castle Hound
- [x] **Turnip Patch Plots & Bubbling Stew Pot (`src/economy.rs`):** Two raised-bed Turnip Patch Plots (`ExteriorShell` borders/beds + purple-top turnip props) and a bubbling Homestead Stew Pot & Campfire (`STEW_POT_POS`) with flickering campfire light and sitting logs.
- [x] **Cozy Needs & Stew Pot Morale Loop (`src/economy.rs`, `src/npc/mod.rs`):** `CozyNeeds` component (`energy`, `stew_buff_timer`, `quirky_trait` such as *Turnip Connoisseur*, *Afraid of Heights*, *Whistles Loudly*, *Self-Taught Architect*, *Nap Enthusiast*). When energy dips below 25, peasants stroll to the Stew Pot, sit for 4 seconds eating warm turnip stew, and gain `"Well-Fed & Cheerful (+50% Speed)"`.
- [x] **Chain of Command: The Reeve & Master Mason (`src/npc/overseers.rs`):** Spawned **Rowan the Reeve** (green coat & wide-brimmed hat patrolling Turnip Patch, Trees, and Stockpile) and **Master Mason Cedric** (blue tunic, guild apron & drafting board supervising blueprints, emitting a `+65%` build aura and automatically summoning scaffolding on nearby stakes).
- [x] **Reeve's Priority Decree `[R]` (`src/npc/overseers.rs`, `src/ui.rs`):** Hotkey `[R]` and HUD button cycling between `Balanced Village`, `All-Hands Timber & Quarrying`, `All-Hands Castle Construction`, and `Festival & Feast Rest (Everyone gathers at campfire for 2x morale)`.
- [x] **The Loyal Castle Hound & Grazing Sheep (`src/pets.rs`):** Spawned **Barnaby's Bane** (trots behind High Lord Aldric, wags tail, barks happily to cheer nearby peasants, and jumps when petted with `[F]` in First-Person Mind-Meld mode to restore Ego & Whisper Influence to 100%) plus 4 fluffy **Grazing Sheep** wandering and munching grass.

---

## Milestone 4 / Autonomous Sprint — Step 4 of 5: Cozy Seasons, Warm Nighttime Braziers & Non-Punishing Slapstick Skirmishes
- [x] **Cozy Day/Night & 4-Season Cycle (`src/atmosphere.rs`):** Gentle lighting cycle (`SunlitDay`, `GoldenHourDusk`, `CozyBrazierNight`, `RosyDawn`) and 4-Season grass/foliage tinting (`GoldenSummer`, `CrispAutumn`, `CozyFrost`, `BloomingSpring`) controlled via hotkey **`[T]` ("Toggle Cozy Time / Season")**.
- [x] **Warm Nighttime Castle Braziers (`src/atmosphere.rs`):** Automatically mounts and ignites warm flickering `CastleBrazierLight` point-lights on Gatehouses, Towers, and wall bastions at Dusk/Night so the castle glows like a warm miniature diorama.
- [x] **Player-Paced Slapstick "Monty Python" Skirmishes (`src/raiders.rs`):** Defaults to **Peaceful Sandbox Mode**; pressing **`[B]` ("Blow Challenge Horn / Summon Petty Bandits")** rings the physical **Homestead Warning Bell** (`ExteriorShell`) and spawns 4 comical Saucepan-Helmet **Turnip Bandits** shouting silly taunts.
- [x] **Non-Destructive Turnip Defense (`src/raiders.rs`):** Peasants toss flying turnips (`TossedTurnipProjectile`) while High Lord Aldric, Sir Barnaby, and Barnaby's Bane charge out; bonked bandits never destroy player structures—they spin comically, drop bonus Timber Logs + Turnips, shout *"RUN AWAY! RUN AWAY!"*, and flee while cheering the village!

---

## Backburner Bonus / Autonomous Sprint — Step 5 of 5: Watertight `.STL` 3D Printer Exporter, UI Polish & Full Codebase Audit
- [x] **`[F12]` Binary `.STL` Exporter (`src/stl_export.rs`):** Pure-Rust binary `.STL` writer that collects all world-transformed triangles exclusively from `ExteriorShell` and `TerrainPlinth` meshes (ignoring NPCs, trees, stakes, scaffolding, sheep, and temporary props), computes outward normals, and writes `exported_castle_diorama.stl` with celebratory HUD banner feedback.
- [x] **Unit Test Verification (`src/stl_export.rs`):** `test_binary_stl_export_format` unit test verifying binary `.STL` header, little-endian triangle count, byte length (`80 + 4 + 50 * N`), and outward normal computation.
- [x] **"Castle Chronicle Tapestry" & Collapsible `[H]` Controls Help Card (`src/ui/mod.rs`, `src/ui/briefing.rs`):** Styled HUD in medieval parchment (`Color::srgb(0.95, 0.90, 0.80)`) and dark wood tones with a collapsible `[H]` Controls Cheat-Sheet card and `[F12]` export button.
- [x] **Full Autonomous Codebase Audit:** All `.rs` source files verified strictly under 350 lines with zero warnings and zero errors across `cargo check --all-targets` and `cargo test`.

---

## Core Design Upgrade: Autonomous NPC Architect, "The Unseen Whim" & Pre-Work Briefing
- [x] **Default Tool = `[1] Observe & Inspect` + Context Hover Tooltip + Clickable Bottom Toolbar (`src/blueprint/mod.rs`, `src/ui/mod.rs`, `src/state.rs`):** Starts in `DirectorMode::Inspect` (`Observe & Inspect [1]`) so left-drag pans the tabletop camera and hovering over any NPC or structure shows a Context Tooltip Card (`HoverTooltipState`). Added clickable bottom Bevy `Button` toolbar (`[1]` Observe & Inspect, `[2]` Subconscious Whim, `[3]` Nudge/Summon Helper, `[V]` Enter Lord's Mind, `[Space]` Sound Work Horn / Pause & Resume).
- [x] **Autonomous NPC Architect System (`src/npc/architect.rs`):** High Lord Aldric, Sir Barnaby, and Master Mason Cedric autonomously survey and stake out new castle phases every ~14s (Northern Palisade Arc -> Southern Gatehouse -> East Flank Watchtower Keep -> Western Curtain Wall -> Outer Concentric Curtains), or on demand via **`[G]` ("Propose Next Blueprint Now")**.
- [x] **"The Unseen Whim" — Unquestioning Servant Acceptance (`src/blueprint/mod.rs`, `src/npc/mod.rs`):** Whenever the player uses `[2] Subconscious Whim` to add stakes, carve a gatehouse, or Right-Click erase NPC-laid stakes, servants seamlessly redirect to the new line without error and treat every modification as their own/Master's brilliant idea.
- [x] **Pre-Work Council Briefing & Live Era Objective Tracker (`src/state.rs`, `src/ui/briefing.rs`, `src/ui/mod.rs`):** Starts in `SimulationPace::PreWorkBriefing` with the **"Royal Director's Briefing & Era Objective"** modal (`[H]` / `[Tab]`) and a persistent top-right Era Objective & Autonomous Architect countdown tracker.

---

## Autonomous NPC Architect Refinement: Concentric Master Plan, Flawed Lords, Veto Memory & In-Place Retrofitting
- [x] **Concentric "Master Plan Ring" Architecture (`src/npc/architect.rs`):** Structured Concentric Master Plan centered on `Vec3::ZERO` across the **Inner Ring (`11.0m`)**, **Middle Ring (`20.0m`)**, and **Outer Ring (`29.0m`)**; each proposal claims the next unbuilt, non-vetoed sector (`North Arc`, `East Arc`, `South Arc`, `West Arc`) and snaps cleanly to neighboring wall endpoints.
- [x] **"Charmingly Flawed" Lord Personalities (`src/npc/architect.rs`, `src/mind_meld.rs`):** Alternates blueprint proposals between **High Lord Aldric (`PracticalFarmerLord`)** (stakes low fences with wide Cart Gatehouses, skipping towers) and **Sir Barnaby (`ParanoidMartialVassal`)** (stakes thick walls and Corner Watchtowers, refusing to leave a Gatehouse with a HUD/Chronicle tip inviting the player to carve an archway with `[2]` or whisper sense into him with `[V] -> [Z]`).
- [x] **Zero-Tedium "Subconscious Veto Memory" (`VetoedZones` in `src/state.rs`, `src/blueprint/mod.rs`, `src/npc/architect.rs`):** Right-Click Scrubber in `[2]` records erased spots in `VetoedZones` so NPC Architects **never** re-stake player-erased courtyards (*"On second thought, that spot had terrible feng shui. I shall leave it open as a courtyard!"*).
- [x] **Seamless Mid-Haul Re-Binding & Sustainable Cozy Trees (`src/npc/mod.rs`, `src/world.rs`):** Peasants carrying logs to erased/modified stakes keep the log in their hands and redirect to the nearest active stake; felled trees spawn a `TreeSapling` stump that regrows into a full harvestable tree after `75.0` seconds.
- [x] **In-Place Era Retrofitting (Timber -> Stone Evolution) (`src/blueprint/shell_mesh.rs`, `src/blueprint/mod.rs`, `src/npc/overseers.rs`):** In Era 3+, **Master Mason Cedric** (or the player dragging `[2]` over an existing wooden wall) upgrades existing timber palisade segments **in-place** into Norman Ashlar Stone Walls (`ExteriorShell`).
- [x] **Organic "Feudal Architectural Debates" & Unseen Tie-Breakers (`src/npc/debates.rs`, `src/npc/architect.rs`, `src/ui/briefing.rs`, `src/ui/mod.rs`):** Every 2nd autonomous ring cycle triggers an `ActiveFeudalDebate` where High Lord Aldric and Sir Barnaby face each other (~2.5m apart inside a golden/crimson gizmo ring) with animated debate gestures, floating 2D speech bubbles, and a HUD Debate Card; auto-resolves hands-off after 15s OR resolves immediately when the player Left-Clicks either Lord or presses `[Z]` (Back Aldric) / `[X]` (Back Barnaby) as an Unseen Whim tie-breaker.
