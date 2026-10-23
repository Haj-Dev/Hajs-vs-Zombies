# Scene system

The game is split into **scenes** (title, level, menu, ...) driven by a small
stack-based `SceneManager` in `src/scene.rs`. Only the scene on top of the stack
is updated and drawn; the ones below stay alive so they can be returned to.

## The `Scene` trait

```rs
pub trait Scene {
    fn update(&mut self, input: &Input) -> Transition;

    fn draw_top<'s>(
        &mut self,
        target: Target<'s, TopScreen, ReadyMode>,
    ) -> Result<Target<'s, TopScreen, ReadyMode>, Error>;

    fn draw_bottom<'s>(
        &mut self,
        target: Target<'s, BottomScreen, ReadyMode>,
    ) -> Result<Target<'s, BottomScreen, ReadyMode>, Error>;
}
```

The draw methods take concrete screen types, so the trait stays object-safe and
scenes can be stored as `Box<dyn Scene>`.

## Transitions

`update` returns what the manager should do next:

```rs
pub enum Transition {
    None,                  // stay on this scene
    Push(Box<dyn Scene>),  // stack a new scene on top (e.g. a pause menu)
    Pop,                   // drop this scene, revealing the one below
    Quit,                  // close the game
}
```

## The manager

`SceneManager` owns a `Vec<Box<dyn Scene>>` and routes everything to
`last_mut()` — the top of the stack:

```rs
pub struct SceneManager {
    stack: Vec<Box<dyn Scene>>,
}

impl SceneManager {
    pub fn new(initial: Box<dyn Scene>) -> Self {
        Self { stack: vec![initial] }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn update(&mut self, input: &Input) {
        let Some(scene) = self.stack.last_mut() else {
            return;
        };
        let transition = scene.update(input);

        match transition {
            Transition::None => {}
            Transition::Push(scene) => self.stack.push(scene),
            Transition::Pop => {
                self.stack.pop();
            }
            Transition::Quit => self.stack.clear(),
        }
    }

    pub fn draw_top<'s>(
        &mut self,
        target: Target<'s, TopScreen, ReadyMode>,
    ) -> Result<Target<'s, TopScreen, ReadyMode>, Error> {
        match self.stack.last_mut() {
            Some(scene) => scene.draw_top(target),
            None => Ok(target),
        }
    }

    pub fn draw_bottom<'s>(
        &mut self,
        target: Target<'s, BottomScreen, ReadyMode>,
    ) -> Result<Target<'s, BottomScreen, ReadyMode>, Error> {
        match self.stack.last_mut() {
            Some(scene) => scene.draw_bottom(target),
            None => Ok(target),
        }
    } 
}
```

## Frame loop

`App` holds the manager; `State::main` drives it:

```rs
while self.handles.apt.main_loop() {
    self.handles.hid.scan_input();
    let input = Input::new(self.handles.hid);

    self.app.scenes.update(&input);
    if self.app.scenes.is_empty() {
        break;
    }

    top_target = self.app.scenes.draw_top(top_target)?;
    bottom_target = self.app.scenes.draw_bottom(bottom_target)?;

    self.handles.gfx.wait_for_vblank();
}
```

## Behaviour

- **Only the top scene is active.** Lower scenes are not updated and not drawn,
  but they are *not* dropped — their state is preserved.
- Each scene clears its target, so lower scenes are never visible through the
  top one.
- `Pop` therefore resumes the scene below exactly where it left off (the
  pause-menu pattern).

## Adding a scene

1. Create `src/scenes/<scene_name>.rs` with a struct implementing `Scene`.
2. Register it in `src/scenes/mod.rs`.
3. Return `Transition::Push(Box::new(MyScene::new()))` from whichever scene
   should open it.
4. Return `Transition::Pop` from `update` when the scene should close.
