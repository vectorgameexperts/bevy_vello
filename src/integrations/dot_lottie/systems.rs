use super::DotLottiePlayer;
use crate::{
    integrations::lottie::{
        asset::{VelloLottie, VelloLottieHandle},
        PlaybackPlayMode,
    },
    PlaybackDirection, PlaybackLoopBehavior, PlaybackOptions, PlayerTransition, Playhead,
};
use bevy::{prelude::*, utils::Instant};
use std::time::Duration;
use vello_svg::usvg::strict_num::Ulps;

/// Advance all the dotLottie playheads in the scene
pub fn advance_dot_lottie_playheads(
    mut query: Query<(
        &VelloLottieHandle,
        &mut Playhead,
        &mut DotLottiePlayer,
        &PlaybackOptions,
    )>,
    mut assets: ResMut<Assets<VelloLottie>>,
    time: Res<Time>,
) {
    for (asset_handle, mut playhead, mut player, options) in query.iter_mut() {
        // Get asset
        let Some(asset) = assets.get_mut(asset_handle.id()) else {
            continue;
        };

        // Keep playhead bounded
        let start_frame = options.segments.start.max(asset.composition.frames.start);
        let end_frame = options
            .segments
            .end
            .min(asset.composition.frames.end)
            .prev();
        playhead.frame = playhead.frame.clamp(start_frame, end_frame);

        // Check if we are stopped
        if player.stopped {
            continue;
        }

        // Set first render
        playhead.first_render.get_or_insert(Instant::now());

        // Auto play
        if !player.started && options.autoplay {
            player.started = true;
            player.playing = true;
        }
        // Return if paused
        if !player.playing {
            continue;
        }

        // Handle intermissions
        if let Some(ref mut intermission) = playhead.intermission {
            intermission.tick(time.delta());
            if intermission.finished() {
                playhead.intermission.take();
                match options.direction {
                    PlaybackDirection::Normal => {
                        playhead.frame = start_frame;
                    }
                    PlaybackDirection::Reverse => {
                        playhead.frame = end_frame;
                    }
                }
            }
            continue;
        }

        // Advance playhead
        let length = end_frame - start_frame;
        playhead.frame += (time.delta_secs_f64()
            * options.speed
            * asset.composition.frame_rate
            * (options.direction as i32 as f64)
            * playhead.playmode_dir)
            % length;

        // Keep the playhead bounded between segments
        let looping = match options.looping {
            PlaybackLoopBehavior::Loop => true,
            PlaybackLoopBehavior::Amount(amt) => playhead.loops_completed < amt,
            PlaybackLoopBehavior::DoNotLoop => false,
        };
        if playhead.frame > end_frame {
            if looping {
                playhead.loops_completed += 1;
                if let PlaybackPlayMode::Bounce = options.play_mode {
                    playhead.playmode_dir *= -1.0;
                }
                // Trigger intermission, if applicable
                if options.intermission > Duration::ZERO {
                    playhead
                        .intermission
                        .replace(Timer::new(options.intermission, TimerMode::Once));
                    playhead.frame = end_frame;
                } else {
                    // Wrap around to the beginning of the segment
                    playhead.frame = start_frame + (playhead.frame - end_frame);
                }
            } else {
                playhead.frame = end_frame;
            }
            // Obey play mode
            if let PlaybackPlayMode::Bounce = options.play_mode {
                playhead.frame = end_frame;
            }
        } else if playhead.frame < start_frame {
            if looping {
                playhead.loops_completed += 1;
                if let PlaybackPlayMode::Bounce = options.play_mode {
                    playhead.playmode_dir *= -1.0;
                }
                // Trigger intermission, if applicable
                if options.intermission > Duration::ZERO {
                    playhead
                        .intermission
                        .replace(Timer::new(options.intermission, TimerMode::Once));
                    playhead.frame = start_frame;
                } else {
                    // Wrap around to the beginning of the segment
                    playhead.frame = end_frame - (start_frame - playhead.frame);
                }
            } else {
                playhead.frame = start_frame;
            }
            // Obey play mode
            if let PlaybackPlayMode::Bounce = options.play_mode {
                playhead.frame = start_frame;
            }
        }
    }
}

pub fn run_transitions(
    mut query_player: Query<(
        &mut DotLottiePlayer,
        &Playhead,
        &PlaybackOptions,
        &GlobalTransform,
        &mut VelloLottieHandle,
    )>,
    mut assets: ResMut<Assets<VelloLottie>>,
    windows: Query<&Window>,
    query_view: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut hovered: Local<bool>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let Ok((camera, view)) = query_view.get_single() else {
        return;
    };

    let pointer_pos = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(view, cursor).ok())
        .map(|ray| ray.origin.truncate());

    for (mut player, playhead, options, gtransform, current_asset_handle) in query_player.iter_mut()
    {
        if player.stopped {
            continue;
        }

        let current_asset = assets
            .get_mut(current_asset_handle.id())
            .unwrap_or_else(|| {
                panic!(
                    "asset not found for state: '{}'",
                    player.current_state.unwrap()
                )
            });

        let is_inside = {
            match pointer_pos {
                Some(pointer_pos) => {
                    let local_transform = current_asset
                        .local_transform_center
                        .compute_matrix()
                        .inverse();
                    let transform = gtransform.compute_matrix() * local_transform;
                    let mouse_local = transform
                        .inverse()
                        .transform_point3(pointer_pos.extend(0.0));
                    mouse_local.x <= current_asset.width
                        && mouse_local.x >= 0.0
                        && mouse_local.y >= -current_asset.height
                        && mouse_local.y <= 0.0
                }
                None => false,
            }
        };

        for transition in player.state().transitions.iter() {
            match transition {
                PlayerTransition::OnAfter { state, secs } => {
                    let started = playhead.first_render;
                    if started.is_some_and(|s| s.elapsed().as_secs_f32() >= *secs) {
                        player.next_state = Some(state);
                        break;
                    }
                }
                PlayerTransition::OnComplete { state } => {
                    // can be irrefutable if only feature is lottie
                    let loops_needed = match options.looping {
                        PlaybackLoopBehavior::DoNotLoop => Some(0),
                        PlaybackLoopBehavior::Amount(amt) => Some(amt),
                        PlaybackLoopBehavior::Loop => Some(0),
                    };
                    match options.direction {
                        PlaybackDirection::Normal => {
                            let end_frame = options
                                .segments
                                .end
                                .min(current_asset.composition.frames.end)
                                .prev();
                            if playhead.frame == end_frame
                                && loops_needed
                                    .is_some_and(|needed| playhead.loops_completed >= needed)
                            {
                                player.next_state = Some(state);
                                break;
                            }
                        }
                        PlaybackDirection::Reverse => {
                            let start_frame = options
                                .segments
                                .start
                                .max(current_asset.composition.frames.start);
                            if playhead.frame == start_frame
                                && loops_needed
                                    .is_some_and(|needed| playhead.loops_completed >= needed)
                            {
                                player.next_state = Some(state);
                                break;
                            }
                        }
                    }
                }
                PlayerTransition::OnMouseEnter { state } => {
                    if is_inside {
                        player.next_state = Some(state);
                        *hovered = true;
                        break;
                    }
                }
                PlayerTransition::OnMouseClick { state } => {
                    if is_inside && buttons.just_pressed(MouseButton::Left) {
                        player.next_state = Some(state);
                        break;
                    }
                }
                PlayerTransition::OnMouseLeave { state } => {
                    if *hovered && !is_inside {
                        player.next_state = Some(state);
                        *hovered = false;
                        break;
                    } else if is_inside {
                        *hovered = true;
                    }
                }
                PlayerTransition::OnShow { state } => {
                    if playhead.first_render.is_some() {
                        player.next_state = Some(state);
                        break;
                    }
                }
            }
        }
    }
}

pub fn transition_state(
    mut commands: Commands,
    mut query_sm: Query<(Entity, &mut DotLottiePlayer, &mut Playhead)>,
    assets: Res<Assets<VelloLottie>>,
) {
    for (entity, mut player, mut playhead) in query_sm.iter_mut() {
        // Is there a state to transition to?
        let Some(next_state) = player.next_state else {
            continue;
        };
        // Is it the same state?
        if Some(next_state) == player.current_state {
            player.next_state.take();
            continue;
        }

        info!("animation controller transitioning to={next_state}");
        let target_state = player
            .states
            .get(&next_state)
            .unwrap_or_else(|| panic!("state not found: '{}'", next_state));
        let target_options = target_state
            .options
            .as_ref()
            .or(player.state().options.as_ref())
            .cloned()
            .unwrap_or_default();

        // Swap asset
        if let Some(target_handle) = target_state.asset.as_ref() {
            commands.entity(entity).insert(target_handle.clone());
        }
        // Reset playheads if requested
        let reset_playhead =
            player.state().reset_playhead_on_exit || target_state.reset_playhead_on_start;
        if reset_playhead {
            let target_asset = target_state.asset.as_ref();
            if let Some(target_asset) = target_asset {
                let Some(asset) = assets.get(target_asset.id()) else {
                    warn!("not ready for state transition, re-queueing {next_state}...");
                    player.next_state = Some(next_state);
                    continue;
                };
                let frame = match target_options.direction {
                    PlaybackDirection::Normal => target_options
                        .segments
                        .start
                        .max(asset.composition.frames.start),
                    PlaybackDirection::Reverse => target_options
                        .segments
                        .end
                        .min(asset.composition.frames.end)
                        .prev(),
                };
                playhead.seek(frame);
            }
        }

        // Swap theme
        if let Some(theme) = target_state.theme.as_ref() {
            commands.entity(entity).insert(theme.clone());
        }

        // Swap playback options
        if target_state.options.is_some() {
            commands.entity(entity).insert(target_options);
        }

        // Reset playhead state
        playhead.intermission.take();
        playhead.loops_completed = 0;
        playhead.first_render.take();
        playhead.playmode_dir = 1.0;

        // Reset player state
        player.started = false;
        player.playing = false;
        player.current_state.replace(next_state);
    }
}
