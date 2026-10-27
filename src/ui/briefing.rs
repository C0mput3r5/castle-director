use crate::ui::{
    spawn_hud_button, BriefingModalPanel, ContextTooltipPanel, ContextTooltipText,
    FeudalDebateCardPanel, FeudalDebateCardText, HudButtonAction, MindMeldBannerText,
    MindMeldMonologueText, MindMeldPanel, StlBannerPanel, StlBannerText, DARK_WOOD_BG,
    PARCHMENT_BORDER, PARCHMENT_TEXT,
};
use bevy::prelude::*;

pub fn spawn_center_modals_and_tooltips(root: &mut ChildBuilder) {
    // 1. PRE-WORK BRIEFING & ROYAL CHARTER MODAL (Visible at startup in PreWorkBriefing, togglable with [H] or [Tab])
    root.spawn((
        BriefingModalPanel,
        Node {
            width: Val::Px(820.0),
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(18.0)),
            row_gap: Val::Px(8.0),
            border: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.13, 0.10, 0.07, 0.96)),
        BorderColor(Color::srgb(0.95, 0.80, 0.35)),
        BorderRadius::all(Val::Px(10.0)),
    ))
    .with_children(|modal| {
        modal.spawn((
            Text::new("ROYAL DIRECTOR'S BRIEFING & ERA OBJECTIVE ([H] or [Tab] Toggle Charter)"),
            TextFont { font_size: 18.5, ..default() },
            TextColor(Color::srgb(1.0, 0.88, 0.38)),
        ));
        modal.spawn((
            Text::new(
                "YOUR ROLE — THE UNSEEN GUIDING HAND:\n\
                 You do NOT have to build the castle yourself! Your Lords and Master Mason will autonomously design, stake out, and build the castle phase by phase.\n\
                 • Default Mode is [1] Observe & Inspect: Pan the tabletop camera with Left-Drag/WASD, hover over any villager or building to inspect it, or click a stake to ring the Priority Bell.\n\
                 • Whenever you use [2] Subconscious Whim to add stakes, carve a gatehouse, or Right-Drag erase stakes, your servants have ZERO awareness of an outside player—they treat every change as their own brilliant idea and obey without question!"
            ),
            TextFont { font_size: 13.0, ..default() },
            TextColor(PARCHMENT_TEXT),
        ));
        modal.spawn((
            Text::new(
                "WHAT DOES WHAT (HOMESTEAD LEGEND):\n\
                 • High Lord Aldric & Master Mason Cedric: Autonomously stake out new castle walls, gatehouses & towers every ~14s (or press [G] anytime) and emit speed auras.\n\
                 • Rowan the Reeve ([R]): Manages village priorities between Timber, Construction, and Campfire Feast.\n\
                 • Loyal Peasants ([3] Recruit): Fell oak trees, haul logs, dig ditches (Stage 1), and erect watertight castle walls (Stage 2).\n\
                 • Farmhouse & Bubbling Stew Pot: Tired servants stroll to the campfire for warm turnip stew, gaining 'Well-Fed & Cheerful (+50% Speed)'.\n\
                 • Timber Stockpile: Stores felled oak logs ready for castle construction."
            ),
            TextFont { font_size: 12.4, ..default() },
            TextColor(Color::srgb(0.84, 0.94, 0.82)),
        ));
        modal.spawn((
            Node {
                margin: UiRect::top(Val::Px(4.0)),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            },
        ))
        .with_children(|row| {
            spawn_hud_button(
                row,
                ">>> Sound the Work Horn / Begin the Day's Work [Space] <<<",
                HudButtonAction::ToggleWorkPace,
            );
            spawn_hud_button(
                row,
                "[G] Have Lord & Mason Propose Next Blueprint Now",
                HudButtonAction::ProposeNextBlueprint,
            );
            spawn_hud_button(
                row,
                "[H] Hide Briefing Card",
                HudButtonAction::ToggleHelpCard,
            );
        });
    });

    // 2. CONTEXT HOVER TOOLTIP CARD (Shown when hovering in [1] Observe & Inspect Mode)
    root.spawn((
        ContextTooltipPanel,
        Node {
            width: Val::Px(660.0),
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(11.0)),
            row_gap: Val::Px(4.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(DARK_WOOD_BG),
        BorderColor(Color::srgb(0.96, 0.84, 0.42)),
        BorderRadius::all(Val::Px(8.0)),
        Visibility::Hidden,
        PickingBehavior::IGNORE,
    ))
    .with_children(|tip| {
        tip.spawn((
            ContextTooltipText,
            Text::new("Hover over any Villager, Lord, Stake, or Building in [1] Observe Mode..."),
            TextFont { font_size: 12.8, ..default() },
            TextColor(PARCHMENT_TEXT),
        ));
    });

    // 3. FEUDAL ARCHITECTURAL DEBATE IN PROGRESS CARD (Non-intrusive center-top card with tie-breaker buttons)
    root.spawn((
        FeudalDebateCardPanel,
        Node {
            width: Val::Px(690.0),
            align_self: AlignSelf::Center,
            display: Display::None,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(12.0)),
            row_gap: Val::Px(6.0),
            border: UiRect::all(Val::Px(2.5)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.16, 0.11, 0.08, 0.94)),
        BorderColor(Color::srgb(0.98, 0.82, 0.28)),
        BorderRadius::all(Val::Px(9.0)),
    ))
    .with_children(|deb| {
        deb.spawn((
            FeudalDebateCardText,
            Text::new("FEUDAL ARCHITECTURAL DEBATE IN PROGRESS..."),
            TextFont { font_size: 12.8, ..default() },
            TextColor(PARCHMENT_TEXT),
        ));
        deb.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                ..default()
            },
        ))
        .with_children(|btns| {
            spawn_hud_button(
                btns,
                "[Z / Click Aldric] Back High Lord Aldric (Gatehouse & Cart Fence)",
                HudButtonAction::BackAldricDebate,
            );
            spawn_hud_button(
                btns,
                "[X / Click Barnaby] Back Sir Barnaby (Watchtower & Solid Wall)",
                HudButtonAction::BackBarnabyDebate,
            );
        });
    });

    // 4. CELEBRATORY .STL EXPORT BANNER
    root.spawn((
        StlBannerPanel,
        Node {
            align_self: AlignSelf::Center,
            padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
            border: UiRect::all(Val::Px(2.5)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.16, 0.24, 0.12, 0.95)),
        BorderColor(Color::srgb(0.98, 0.86, 0.35)),
        BorderRadius::all(Val::Px(8.0)),
        Visibility::Hidden,
        PickingBehavior::IGNORE,
    ))
    .with_children(|b| {
        b.spawn((
            StlBannerText,
            Text::new("Royal Charter & 3D Printable Diorama saved!"),
            TextFont { font_size: 15.5, ..default() },
            TextColor(Color::srgb(1.0, 0.92, 0.48)),
        ));
    });

    // 4. LORD MIND-MELD HUD (Visible in First-Person Mode [V])
    root.spawn((
        MindMeldPanel,
        Node {
            width: Val::Px(770.0),
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(14.0)),
            row_gap: Val::Px(6.0),
            border: UiRect::all(Val::Px(2.5)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.14, 0.09, 0.18, 0.92)),
        BorderColor(PARCHMENT_BORDER),
        BorderRadius::all(Val::Px(10.0)),
        Visibility::Hidden,
        PickingBehavior::IGNORE,
    ))
    .with_children(|meld| {
        meld.spawn((
            MindMeldBannerText,
            Text::new("MIND-MELD: Inside the mind of High Lord Aldric ([Tab] Switch Lord | [V] Return to Sky)"),
            TextFont { font_size: 16.5, ..default() },
            TextColor(Color::srgb(1.0, 0.88, 0.38)),
        ));
        meld.spawn((
            MindMeldMonologueText,
            Text::new("Inner Monologue: ..."),
            TextFont { font_size: 14.0, ..default() },
            TextColor(PARCHMENT_TEXT),
        ));
        meld.spawn((
            Text::new(
                "WHISPER INCEPTION & ROYAL HOUND HOTKEYS:\n\
                 • [Z] Whisper Urgency (\"Bandits could attack before sundown!\" -> 2x Build Speed Aura)\n\
                 • [X] Whisper Praise  (\"Workers deserve warm turnip stew...\" -> Morale & Chop Boost)\n\
                 • [C] Whisper Rivalry (\"Is that other Lord slacking off?!\" -> Feudal Debate & 2x Aura Radius)\n\
                 • [F] Pet the Castle Hound (\"Barnaby's Bane jumps happily!\" -> Restores Ego & Whisper Influence to 100%!)"
            ),
            TextFont { font_size: 12.8, ..default() },
            TextColor(Color::srgb(0.68, 0.94, 0.78)),
        ));
    });
}

