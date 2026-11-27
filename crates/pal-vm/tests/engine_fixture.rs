use pal_vm::{Engine, EngineConfig};

#[test]
fn engine_new_starts_without_diagnostic_visuals() {
    let engine = Engine::new(EngineConfig::default()).expect("engine should boot without root");

    assert_eq!(engine.sprites().len(), 0);
    assert_eq!(engine.sprites().surface_count(), 0);
    assert_eq!(engine.sprites().render_node_count(), 0);
    assert_eq!(engine.sprites().commands().len(), 0);
    assert_eq!(engine.task_system().active_task_count(), 0);
}

#[test]
fn pal_debug_does_not_create_runtime_objects() {
    std::env::set_var("PAL_DEBUG", "1");
    let engine = Engine::new(EngineConfig::default()).expect("engine should boot without root");
    std::env::remove_var("PAL_DEBUG");

    assert_eq!(engine.sprites().len(), 0);
    assert_eq!(engine.sprites().surface_count(), 0);
    assert_eq!(engine.sprites().render_node_count(), 0);
    assert_eq!(engine.sprites().commands().len(), 0);
    assert_eq!(engine.task_system().active_task_count(), 0);
}

#[test]
fn crossfade_inherits_mode_eleven_in_software_and_deferred_scenes() {
    use pal_vm::scene::{rasterize_scene_rgba, DrawCommand};
    use pal_vm::sprite::{PalColor, PalRenderMode, PalVec3};
    use std::time::Duration;

    for gpu in [false, true] {
        let mut engine = Engine::new(EngineConfig::default()).unwrap();
        engine.set_host_gpu_renderer(gpu);
        let sprites = engine.sprites_mut();
        sprites.set_registered_profile(true);
        let mode_setter = sprites
            .create_rgba_sprite(1, 1, vec![255; 4], PalVec3::new(4, 0, 0), -1, "mode setter")
            .unwrap();
        assert!(sprites.set_render_mode(mode_setter, PalRenderMode::new(11)));
        let old = sprites
            .create_rgba_sprite(1, 1, vec![255, 0, 0, 128], PalVec3::default(), 0, "old")
            .unwrap();
        let new = sprites
            .create_rgba_sprite(1, 1, vec![0, 255, 0, 128], PalVec3::default(), 0, "new")
            .unwrap();
        sprites.set_color(old, PalColor(0xff00_0000));
        sprites.set_color(new, PalColor(0xff00_0000));
        let transition = sprites.create_transition_handle();
        assert!(sprites.set_transition(transition, 1, Some(new), Some(old), 1, 0, 100));

        let scene = engine.update_with_delta(Duration::from_millis(50)).unwrap().scene;
        let DrawCommand::Sprite(fade) = &scene.commands[1] else {
            panic!("expected the combined transition draw");
        };
        assert_eq!(fade.effective_blend_state().pipeline_index(fade.render_mode), 11);
        // The inherited state disables tint and replaces the destination even
        // though both source pixels are partially transparent.
        assert_eq!(&rasterize_scene_rgba(&scene)[..4], &[128, 128, 0, 255]);
    }
}
